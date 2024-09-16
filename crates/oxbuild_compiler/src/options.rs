use std::{
    env, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct CompileOptions {
    root_dir: PathBuf,
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
        Self { root_dir }
    }

    pub(crate) fn resolve<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.root_dir.join(path).canonicalize()
    }
}
