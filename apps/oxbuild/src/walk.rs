use std::ops::Deref;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use ignore::{DirEntry, Error as WalkError, ParallelVisitor, ParallelVisitorBuilder, WalkState};
use oxc::diagnostics::OxcDiagnostic;

use oxbuild_compiler::{compile, CompileOptions, CompiledOutput};

use crate::OxbuildOptions;

pub struct WalkerBuilder {
    options: Arc<OxbuildOptions>,
    compile_options: Arc<CompileOptions>,
    state: Arc<Mutex<State>>,
}
#[derive(Debug, Default)]
struct State {
    errors: Vec<OxcDiagnostic>,
}

impl WalkerBuilder {
    pub fn new(options: OxbuildOptions) -> Self {
        let compile_options = CompileOptions::new(options.root.deref().to_path_buf())
            .with_d_ts(options.isolated_declarations);
        Self {
            compile_options: Arc::new(compile_options),
            options: Arc::new(options),
            state: Default::default(),
        }
    }

    pub fn walk(&mut self, nthreads: usize) {
        let inner = ignore::WalkBuilder::new(&self.options.src)
            // TODO: use ignore to respect tsconfig include/exclude
            .ignore(false)
            .threads(nthreads)
            .hidden(false)
            .build_parallel();

        inner.visit(self);
    }
}

impl<'s> ParallelVisitorBuilder<'s> for WalkerBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(Walker {
            options: Arc::clone(&self.options),
            compile_options: Arc::clone(&self.compile_options),
            state: Arc::clone(&self.state),
        })
    }
}

pub struct Walker {
    options: Arc<OxbuildOptions>,
    compile_options: Arc<CompileOptions>,
    state: Arc<Mutex<State>>,
}

impl Walker {
    const ALLOWED_EXTS: [&'static str; 8] = ["ts", "tsx", "cts", "mts", "js", "jsx", "mjs", "cjs"];

    fn is_allowed_ext<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .map_or(false, |ext| Self::ALLOWED_EXTS.iter().any(|&e| e == ext))
    }

    #[must_use]
    fn compile(&self, path: &Path) -> Option<CompiledOutput> {
        match compile(&self.compile_options, path) {
            Ok(output) => Some(output),
            Err(errors) => {
                let mut state = self.state.lock().unwrap();
                state.errors.extend(errors);
                None
            }
        }
    }

    fn get_output_path_for(&self, dir: &Path) -> PathBuf {
        let rel = dir.strip_prefix(&self.options.src).unwrap();
        self.options.dist.join(rel)
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
        let dts_path = output_path.with_extension("d.ts");
        fs::write(dts_path, declarations).unwrap();

        // foo.d.ts.map
        if let Some(declarations_map) = declarations_map {
            let map_path = output_path.with_extension("d.ts.map");
            fs::write(map_path, declarations_map.to_json_string()).unwrap();
        }

        WalkState::Continue
    }
}
