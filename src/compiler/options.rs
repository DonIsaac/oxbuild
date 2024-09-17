use std::{
    env, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct CompileOptions {
    root_dir: PathBuf,
    /// Emit .d.ts files using isolatedDeclarations.
    d_ts: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        let cwd = env::current_dir().unwrap();
        Self::new(cwd)
    }
}

impl CompileOptions {
    pub fn new(root_dir: PathBuf) -> Self {
        assert!(root_dir.is_dir());
        assert!(root_dir.is_absolute());
        Self {
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
