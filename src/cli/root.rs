use std::{
    env,
    fs::{self, DirEntry},
    ops::Deref,
    path::{Path, PathBuf},
};

use miette::{IntoDiagnostic, Report, Result, WrapErr};
use package_json::PackageJsonManager;

#[derive(Debug)]
pub(crate) struct Root {
    /// Current working directory from where oxbuild  was run.
    cwd: PathBuf,
    /// Path to directory containing nearest `package.json` file.
    ///
    /// [`None`] if this neither the cwd nor any of its parents contain it.
    root: Option<PathBuf>,
    /// Collected `fs.stat` results from root.
    stat: Vec<DirEntry>,
}

impl Deref for Root {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<Path> for Root {
    fn as_ref(&self) -> &Path {
        self.root.as_ref().unwrap_or(&self.cwd)
    }
}

impl Root {
    /// Create a new [`Root`] with an explicitly provided project directory. If it is a relative
    /// path, it will be joined to the current working directory and canonicalized, but otherwise
    /// left unmodified.
    pub fn new_explicit(project_dir: PathBuf) -> Result<Self> {
        debug_assert!(project_dir.exists(), "project_dir must exist");
        debug_assert!(project_dir.is_dir(), "project_dir must be a directory");

        let cwd = env::current_dir()
            .into_diagnostic()
            .context("Failed to get cwd")?;

        let project_dir = if project_dir.is_absolute() {
            project_dir
        } else {
            cwd.join(project_dir).canonicalize().into_diagnostic()?
        };

        Self::new(cwd, Some(project_dir))
    }

    /// Create a new [`Root`] by looking for the nearest `package.json` file, starting at the cwd.
    pub fn new_inferred() -> Result<Self> {
        let mut manager = PackageJsonManager::new();
        let cwd = env::current_dir()
            .into_diagnostic()
            .context("Failed to get cwd")?;
        let root = manager.locate_closest_from(&cwd).ok().map(|package_json| {
            debug_assert!(package_json.is_file());
            debug_assert_eq!(package_json.file_name().unwrap(), "package.json");
            package_json.parent().unwrap().to_path_buf()
        });

        Self::new(cwd, root)
    }

    fn new(cwd: PathBuf, root: Option<PathBuf>) -> Result<Self> {
        let look_for_configs_in = root.as_ref().unwrap_or(&cwd);

        let stat = fs::read_dir(look_for_configs_in)
            .into_diagnostic()
            .context("Failed to read files in cwd")?;
        let stat: Vec<_> = stat.flatten().collect();

        Ok(Self { cwd, root, stat })
    }

    #[allow(dead_code)]
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    pub fn resolve<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.join(path).canonicalize().into_diagnostic().unwrap()
    }

    pub fn resolve_file<I>(
        &self,
        path: Option<&PathBuf>,
        possible_names: I,
    ) -> Result<Option<PathBuf>>
    where
        I: IntoIterator<Item = &'static str>,
    {
        if let Some(path) = path {
            if !path.exists() {
                return Err(Report::msg(format!("File not found: {}", path.display())));
            }
            if path.is_dir() {
                return Err(Report::msg(format!(
                    "{} is a directory, not a file",
                    path.display()
                )));
            }

            // when the user provides a path to a file, we resolve it from the cwd since they're
            // almost certainly providing paths relative to where they're running the CLI from.
            return self
                .cwd
                .join(path)
                .canonicalize()
                .into_diagnostic()
                .map(Some);
        }

        Ok(self.find(possible_names))
    }

    pub fn find<I>(&self, names: I) -> Option<PathBuf>
    where
        I: IntoIterator<Item = &'static str>,
    {
        for name in names.into_iter() {
            let search_result = self
                .stat
                .iter()
                .filter(|e| e.file_type().map_or(true, |ft| ft.is_file()))
                .find(|entry| entry.file_name().to_str().unwrap() == name);

            if let Some(entry) = search_result {
                return Some(entry.path());
            }
        }

        None
    }
}
