use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use compact_str::CompactString;
use glob::{glob, Paths, Pattern};
use miette::Diagnostic;
use thiserror::{self, Error};

/// A set of glob patterns describing where to find packages in a workspace.
///
/// Supports exclusion patterns (starting with `!`).
#[derive(Debug, Clone)]
pub struct Workspaces {
    pub(super) include_globs: Vec<CompactString>,
    pub(super) exclude_globs: Option<Vec<CompactString>>,
    // nohoist
}
impl<S: AsRef<str>> FromIterator<S> for Workspaces {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let globs = iter.into_iter();
        let hint = globs.size_hint();
        let start_size = hint.1.unwrap_or(hint.0);
        let mut include_globs: Vec<CompactString> = Vec::with_capacity(start_size);
        // let mut exclude_globs: Option<Vec<&'a str>> = None;
        let mut exclude_globs: Vec<CompactString> = Vec::new();

        for glob in globs {
            let glob = glob.as_ref().trim();
            if glob.starts_with('!') {
                exclude_globs.push(glob.strip_prefix('!').unwrap().into());
            } else {
                include_globs.push(glob.into());
            }
        }

        include_globs.shrink_to_fit();
        let exclude = if exclude_globs.is_empty() {
            None
        } else {
            exclude_globs.shrink_to_fit();
            Some(exclude_globs)
        };

        Self {
            include_globs,
            exclude_globs: exclude,
        }
    }
}

impl Workspaces {
    pub fn new(include: Vec<CompactString>, exclude: Option<Vec<CompactString>>) -> Self {
        Self {
            include_globs: include,
            exclude_globs: exclude,
        }
    }

    /// Glob patterns of packages to include in the workspace
    pub fn included(&self) -> &[CompactString] {
        self.include_globs.as_slice()
    }

    /// Glob patterns of packages to exclude from the workspace
    pub fn excluded(&self) -> Option<&[CompactString]> {
        self.exclude_globs.as_deref()
    }

    /// Does this workspace glob list contain globs for included workspaces?
    ///
    /// Does not imply that iteration will yield any paths.
    fn is_empty(&self) -> bool {
        self.include_globs.is_empty()
    }

    /// # Panics
    /// - if `root` is not a directory.
    /// - if `self` is [`empty`].
    ///
    /// [`empty`]: Workspaces::is_empty
    pub fn iter_paths<P: AsRef<Path>>(
        &self,
        root: P,
    ) -> impl Iterator<Item = Result<PathBuf, BadGlobError>> {
        let root = root.as_ref();
        assert!(
            root.is_dir(),
            "Root path {} is not a directory.",
            root.display()
        );
        assert!(!self.is_empty(), "Workspaces list has no included globs.");

        let (iter, errors) = WorkspaceIter::new(root, self);
        let errors_iter = errors.into_iter().map(Err);

        errors_iter.chain(iter.iter())
    }
}

#[derive(Debug)]
struct WorkspaceIter {
    include: Vec<Paths>,
    // needs to be Rc to get around borrow checker in iter
    exclude: Option<Arc<Vec<Pattern>>>,
}

impl WorkspaceIter {
    fn new(root: &Path, workspaces: &Workspaces) -> (Self, Vec<BadGlobError>) {
        let included = workspaces.included();
        let mut include = Vec::with_capacity(included.len());
        let mut errors = Vec::with_capacity(included.len());
        for pattern in included {
            let fullpath = root.join(pattern);
            if let Some(fullpath_str) = fullpath.to_str() {
                let resolved = glob(fullpath_str);
                match resolved {
                    Ok(paths) => include.push(paths),
                    Err(e) => errors.push(BadGlobError::bad_pattern(
                        pattern.to_string(),
                        e,
                        GlobLocation::Include,
                    )),
                }
            } else {
                errors.push(BadGlobError::not_utf8(
                    pattern.to_string(),
                    GlobLocation::Include,
                ));
            }
        }

        let exclude = workspaces.excluded().map(|exclude| {
            let mut exclude_patterns = Vec::with_capacity(exclude.len());
            for raw_pattern in exclude {
                match Pattern::new(raw_pattern) {
                    Ok(pattern) => exclude_patterns.push(pattern),
                    Err(e) => errors.push(BadGlobError::bad_pattern(
                        raw_pattern.to_string(),
                        e,
                        GlobLocation::Exclude,
                    )),
                }
            }
            Arc::new(exclude_patterns)
        });

        (Self { include, exclude }, errors)
    }

    pub(self) fn is_excluded(exclude: Option<&Vec<Pattern>>, path: &Path) -> bool {
        exclude.is_some_and(|exclude| exclude.iter().any(|pat| pat.matches_path(path)))
    }

    // NOTE: not implementing IntoIter because... just look at the size of that type...
    pub fn iter(self) -> impl Iterator<Item = Result<PathBuf, BadGlobError>> {
        let exclude = self.exclude.clone();
        self.include
            .into_iter()
            .flat_map(move |include| {
                let exclude = exclude.clone();
                include.map(move |path| {
                    path.map(|path| {
                        if Self::is_excluded(exclude.as_deref(), path.as_path()) {
                            None
                        } else {
                            Some(path)
                        }
                    })
                    .map_err(|e| BadGlobError::bad_glob(e, GlobLocation::Include))
                })
            })
            .filter_map(|path| match path {
                Err(e) => Some(Err(e)),
                Ok(Some(path)) if path.join("package.json").is_file() => Some(Ok(path)),
                _ => None,
            })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GlobLocation {
    Include,
    Exclude,
}

impl fmt::Display for GlobLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlobLocation::Include => write!(f, "workspace pattern"),
            GlobLocation::Exclude => write!(f, "workspace exclusion pattern"),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum BadGlobError {
    #[error("Invalid {2} '{0}': {1}")]
    Pattern(
        /* glob */ String,
        /* inner */ #[source] glob::PatternError,
        GlobLocation,
    ),
    #[error("{0}")]
    Glob(#[source] glob::GlobError, GlobLocation),
    #[error("Invalid {1} '{0}': pattern is not a UTF-8 string.")]
    NotUtf8(/* glob */ String, GlobLocation),
}
impl BadGlobError {
    fn bad_pattern<S: Into<String>>(
        glob: S,
        inner: glob::PatternError,
        location: GlobLocation,
    ) -> Self {
        Self::Pattern(glob.into(), inner, location)
    }

    fn not_utf8<S: Into<String>>(glob: S, location: GlobLocation) -> Self {
        Self::NotUtf8(glob.into(), location)
    }

    fn bad_glob(glob: glob::GlobError, location: GlobLocation) -> Self {
        Self::Glob(glob, location)
    }
}
