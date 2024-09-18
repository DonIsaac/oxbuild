use std::{
    env, io,
    path::{Path, PathBuf},
};

use oxc::transformer::TransformOptions;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    root_dir: PathBuf,
    /// Emit .d.ts files using isolatedDeclarations.
    d_ts: bool,
    transform_options: TransformOptions,
}

impl Default for CompileOptions {
    fn default() -> Self {
        let cwd = env::current_dir().unwrap();
        Self::new(cwd, TransformOptions::default())
    }
}

impl CompileOptions {
    pub fn new(root_dir: PathBuf, transform_options: TransformOptions) -> Self {
        assert!(root_dir.is_dir());
        assert!(root_dir.is_absolute());

        debug_assert_eq!(root_dir, transform_options.cwd);

        Self {
            transform_options,
            root_dir,
            d_ts: false,
        }
    }

    pub fn with_d_ts(mut self, value: bool) -> Self {
        self.d_ts = value;
        self
    }

    pub(crate) fn resolve<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.root_dir.join(path).canonicalize()
    }
}
