use std::{
    borrow::Cow,
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
};

use glob::glob;
use ignore::{DirEntry, Error as WalkError, ParallelVisitor, ParallelVisitorBuilder, WalkState};
use oxc::diagnostics::{Error, NamedSource, OxcDiagnostic};

use crate::{
    compiler::{CompiledOutput, CompilerOptions, OxBuild},
    workspace::{Package, PackageError, TsConfigStore, Workspace},
    DiagnosticSender,
};

pub struct MonorepoWalker {
    root: Arc<Workspace>,
    nthreads: NonZeroUsize,
}
impl From<Workspace> for MonorepoWalker {
    fn from(root: Workspace) -> Self {
        Self {
            root: Arc::new(root),
            nthreads: std::thread::available_parallelism()
                .unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) }),
        }
    }
}
impl MonorepoWalker {
    pub fn with_nthreads(mut self, nthreads: NonZeroUsize) -> Self {
        self.nthreads = nthreads;
        self
    }

    pub fn walk(self, tsconfigs: &mut TsConfigStore, sender: DiagnosticSender) {
        let nthreads: usize = self.nthreads.into();

        if let Some(workspace_globs) = self.root.workspace_globs() {
            if !workspace_globs.errors.is_empty() {
                let errors: Vec<miette::Error> = workspace_globs
                    .errors
                    .into_iter()
                    .map(|e| {
                        let pos = oxc::span::Span::sized(e.pos as u32, 0);
                        OxcDiagnostic::error(Cow::Owned(e.msg.into()))
                            .with_label(pos)
                            .into()
                    })
                    .collect::<Vec<_>>();

                let root_path = self.root.root_dir.join("package.json");

                sender.send(Some((root_path, errors))).unwrap();
                return;
            }
            let patterns = workspace_globs.patterns;
            assert!(!patterns.is_empty());

            for pattern in patterns {
                for package_root in glob(pattern.as_str()).unwrap() {
                    let Ok(package_root) = package_root else {
                        continue;
                    };
                    if !package_root.is_dir() {
                        continue;
                    }
                    // note: we want diagnostics to use relative dirs cus they're prettier
                    let abs_package_root = self
                        .root
                        .root_dir
                        .join(package_root.clone())
                        .canonicalize()
                        .unwrap();
                    match Package::from_package_dir(tsconfigs, abs_package_root, self.root.clone())
                    {
                        Ok(package) => {
                            debug!("starting walker for package: {package:#?}");
                            let mut walker = WalkerBuilder::new(package, sender.clone());
                            walker.walk(nthreads);
                        }
                        Err(PackageError::NoPackageJson) => continue,
                        Err(PackageError::InvalidPackageJson(e)) => {
                            sender
                                .send(Some((package_root.join("package.json"), vec![e])))
                                .unwrap();
                        }
                        Err(PackageError::InvalidTsConfig(e)) => {
                            sender
                                .send(Some((package_root.join("tsconfig.json"), vec![e])))
                                .unwrap();
                        }
                    }
                }
            }
            return;
        }

        let pkg: Workspace = Arc::try_unwrap(self.root).unwrap();
        let pkg = Package::from(pkg);
        debug!("starting walker for package: {pkg:#?}");
        let mut walker = WalkerBuilder::new(pkg, sender);
        walker.walk(nthreads);
    }
}

pub struct WalkerBuilder {
    options: Arc<CompilerOptions>,
    sender: DiagnosticSender,
    excludes: Vec<String>,
}

impl WalkerBuilder {
    pub fn new(package: Package, sender: DiagnosticSender) -> Self {
        let excludes = package
            .tsconfig()
            .and_then(|t| t.exclude.clone())
            .unwrap_or_default();
        let options = Arc::new(CompilerOptions::from(package));
        Self {
            options,
            sender,
            excludes,
        }
    }

    pub fn walk(&mut self, nthreads: usize) {
        debug!("Starting walker with {} threads", nthreads);
        let mut builder = ignore::WalkBuilder::new(self.options.src());
        // TODO: use ignore to respect tsconfig include/exclude
        builder.ignore(false).threads(nthreads).hidden(false);

        for exclude in self.excludes.iter() {
            builder.add_ignore(exclude);
        }

        let inner = builder.build_parallel();
        inner.visit(self);
    }
}

impl<'s> ParallelVisitorBuilder<'s> for WalkerBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(Walker {
            compiler: OxBuild::new(Arc::clone(&self.options)),
            sender: self.sender.clone(),
        })
    }
}

pub struct Walker {
    compiler: OxBuild,
    sender: DiagnosticSender,
}

impl Walker {
    const ALLOWED_EXTS: [&'static str; 8] = ["ts", "tsx", "cts", "mts", "js", "jsx", "mjs", "cjs"];

    fn is_allowed_ext<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .is_some_and(|ext| Self::ALLOWED_EXTS.iter().any(|&e| e == ext))
    }

    #[must_use]
    fn compile(&mut self, path: &Path) -> Option<CompiledOutput> {
        trace!("Compiling '{}'", path.display());
        let source_text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                let error = OxcDiagnostic::error(format!(
                    "Failed to open source file at '{}': {}",
                    path.display(),
                    e
                ));
                self.sender
                    .send(Some((path.to_path_buf(), vec![Error::new(error)])))
                    .unwrap();
                return None;
            }
        };

        match self.compiler.run(&source_text, path) {
            Ok(output) => Some(output),
            Err(diagnostics) => {
                let source = Arc::new(NamedSource::new(path.to_string_lossy(), source_text));
                let errors = diagnostics
                    .into_iter()
                    .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
                    .collect();
                self.sender
                    .send(Some((path.to_path_buf(), errors)))
                    .unwrap();
                None
            }
        }
    }

    fn get_output_path_for(&self, dir: &Path) -> PathBuf {
        let src = self.compiler.options.src();
        let rel = dir
            .strip_prefix(src)
            .map_err(|_| {
                Error::msg(format!(
                    "Failed to strip prefix '{}' from path '{}'",
                    src.display(),
                    dir.display()
                ))
            })
            .unwrap();
        self.compiler.options.dist().join(rel)
    }
}

impl ParallelVisitor for Walker {
    fn visit(&mut self, entry: Result<DirEntry, WalkError>) -> WalkState {
        let Ok(ent) = entry else {
            return WalkState::Continue;
        };

        // create mirrored path in output directory
        if ent.path().is_dir() {
            let output_dir = self.get_output_path_for(ent.path());
            fs::create_dir_all(&output_dir).unwrap();
            return WalkState::Continue;
        }

        // skip non-js/ts files
        // TODO: copy over json, etc.
        if !Self::is_allowed_ext(ent.path()) {
            return WalkState::Continue;
        }

        // todo: resolve relative paths. Idk if this is absolute or not
        let Some(CompiledOutput {
            source_text,
            source_map,
            declarations,
            declarations_map,
        }) = self.compile(ent.path())
        else {
            return WalkState::Continue;
        };
        let output_path = self.get_output_path_for(ent.path());

        // foo.js
        let js_path = output_path.with_extension("js");
        fs::write(js_path, source_text).unwrap();

        // foo.js.map
        if let Some(source_map) = source_map {
            let map_path = output_path.with_extension("js.map");
            fs::write(map_path, source_map.to_json_string()).unwrap();
        }

        // foo.d.ts
        if let Some(declarations) = declarations {
            let dts_path = output_path.with_extension("d.ts");
            fs::write(dts_path, declarations).unwrap();
        }

        // foo.d.ts.map
        if let Some(declarations_map) = declarations_map {
            let map_path = output_path.with_extension("d.ts.map");
            fs::write(map_path, declarations_map.to_json_string()).unwrap();
        }

        WalkState::Continue
    }
}
