use std::{
    env, io,
    path::{Path, PathBuf},
};

use crate::{
    options::DeclarationsOptions,
    workspace::{Package, TsConfigCompilerOptions},
};

#[derive(Debug)]
pub struct CompileOptions {
    // root_dir: PathBuf,
    /// Emit .d.ts files using isolatedDeclarations.
    declarations_options: Option<DeclarationsOptions>,
    package: Package,
}

impl From<Package> for CompileOptions {
    fn from(package: Package) -> Self {
        let co = package
            .tsconfig
            .as_ref()
            .and_then(|tsconfig| tsconfig.compiler_options());
        let declarations_options = co.and_then(|co| {
            co.isolated_declarations.map(|id| DeclarationsOptions {
                strip_internal: co.strip_internal.unwrap_or(false),
            })
        });
        Self {
            package,
            declarations_options,
        }
    }
}

impl CompileOptions {
    #[inline]
    pub fn declarations_options(&self) -> Option<&DeclarationsOptions> {
        self.declarations_options.as_ref()
    }

    pub(crate) fn resolve<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.package.root_dir.join(path).canonicalize()
    }

    pub fn root_dir(&self) -> &Path {
        &self.package.root_dir
    }
    pub fn src(&self) -> PathBuf {
        self.resolve(
            self.compiler_options()
                .and_then(|co| co.root_dir.as_deref())
                .unwrap_or(Path::new("./src")),
        )
        .unwrap()
    }
    pub fn dist(&self) -> PathBuf {
        self.resolve(
            self.compiler_options()
                .and_then(|co| co.out_dir.as_deref())
                .unwrap_or(Path::new("./src")),
        )
        .unwrap()
    }
    fn compiler_options(&self) -> Option<&TsConfigCompilerOptions> {
        self.package
            .tsconfig
            .as_ref()
            .and_then(|tsconfig| tsconfig.compiler_options())
    }
}
