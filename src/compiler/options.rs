use std::{
    env, io,
    path::{Path, PathBuf},
};

use crate::options::DeclarationsOptions;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    root_dir: PathBuf,
    /// Emit .d.ts files using isolatedDeclarations.
    declarations_options: Option<DeclarationsOptions>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        let cwd = env::current_dir().unwrap();
        Self::new(cwd)
    }
}

impl CompileOptions {
    #[must_use]
    pub fn new(root_dir: PathBuf) -> Self {
        assert!(root_dir.is_dir());
        assert!(root_dir.is_absolute());

        Self {
            root_dir,
            declarations_options: None,
        }
    }

    #[must_use]
    pub fn with_d_ts(mut self, value: Option<DeclarationsOptions>) -> Self {
        self.declarations_options = value;
        self
    }

    #[inline]
    pub fn declarations_options(&self) -> Option<&DeclarationsOptions> {
        self.declarations_options.as_ref()
    }

    pub(crate) fn resolve<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.root_dir.join(path).canonicalize()
    }
}
