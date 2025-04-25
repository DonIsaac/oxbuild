use std::{path::PathBuf, sync::Arc};

use miette::Error;
use package_json::{PackageJson, PackageJsonManager};

use super::{tsconfig::TsConfig, AnyhowWrap, Workspace};

#[derive(Debug)]
pub struct Package {
    pub(crate) package_json: PackageJsonManager,
    pub(crate) root_dir: PathBuf,
    pub(crate) tsconfig: Option<TsConfig>,
    pub(crate) parent: Option<Arc<Workspace>>,
}

#[derive(Debug)]
pub enum PackageError {
    NoPackageJson,
    InvalidPackageJson(Error),
    InvalidTsConfig(Error),
}
impl Package {
    pub fn from_package_dir(
        dir: PathBuf,
        workspace: Arc<Workspace>,
    ) -> std::result::Result<Self, PackageError> {
        debug_assert!(dir.is_dir());

        let mut manager = PackageJsonManager::new();
        let package_json_path = dir.join("package.json");
        if !package_json_path.is_file() {
            return Err(PackageError::NoPackageJson);
        }
        manager.set_file_path(dir.join("package.json"));
        let _pkg = manager
            .read_ref()
            .map_err(|e| PackageError::InvalidPackageJson(Error::new(AnyhowWrap::from(e))))?;

        let tsconfig_path = dir.join("tsconfig.json");
        let tsconfig = tsconfig_path
            .is_file()
            .then(|| TsConfig::from_file(tsconfig_path).map_err(PackageError::InvalidTsConfig))
            .transpose()?;

        Ok(Self {
            package_json: manager,
            root_dir: dir,
            tsconfig,
            parent: Some(workspace),
        })
    }
}

impl PackageInterface for Package {
    fn package_json(&self) -> &PackageJson {
        self.package_json.as_ref()
    }
    fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }
    fn parent(&self) -> Option<&dyn PackageInterface> {
        self.parent
            .as_ref()
            .map(|p| p.as_ref() as &dyn PackageInterface)
    }
    fn tsconfig(&self) -> Option<&TsConfig> {
        self.tsconfig.as_ref()
    }
}

pub trait PackageInterface {
    fn package_json(&self) -> &PackageJson;
    fn root_dir(&self) -> &PathBuf;
    fn tsconfig(&self) -> Option<&TsConfig>;
    fn parent(&self) -> Option<&dyn PackageInterface>;
}
