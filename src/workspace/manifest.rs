use std::{fs, path::PathBuf};

use miette::{Context as _, IntoDiagnostic as _, Result};
use package_json::PackageJson;
use tsconfig::TsConfig;

/// A package manifest.
///
/// May be a single package, a package in a monorepo, or the monorepo root itself.
pub struct Manifest {
    /// absolute
    dir: PathBuf,
    package_json: PackageJson,
    tsconfig: Option<TsConfig>,
}

impl Manifest {
    pub fn new(package_json_path: PathBuf, tsconfig: Option<TsConfig>) -> Result<Self> {
        assert!(
            package_json_path.is_absolute(),
            "package.json paths must be absolute"
        );
        assert!(
            package_json_path
                .file_name()
                .is_some_and(|p| p == "package.json"),
            "Manifest received path to non-package.json: {}",
            package_json_path.display()
        );
        if !package_json_path.is_file() {
            return Err(miette::Report::msg(format!(
                "package.json at {} does not exist",
                package_json_path.display()
            )));
        }
        let package_folder = package_json_path.parent().unwrap().to_path_buf();
        let package_json_raw = fs::read_to_string(&package_json_path)
            .into_diagnostic()
            .with_context(|| {
                format!(
                    "Failed to read package.json at {}",
                    package_json_path.display()
                )
            })?;
        let package_json: PackageJson = serde_json::from_str(&package_json_raw)
            .into_diagnostic()
            .with_context(|| {
            format!(
                "Failed to parse package.json at {}",
                package_json_path.display()
            )
        })?;

        Ok(Self {
            dir: package_folder,
            package_json,
            tsconfig,
        })
    }
}
