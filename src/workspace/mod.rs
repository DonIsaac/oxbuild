mod error;
mod package;
mod tsconfig;

use glob::Pattern;
use glob::PatternError;
use miette::IntoDiagnostic;
use miette::Report;
use miette::Result;
use package::PackageInterface;
use package_json::PackageJson;
use package_json::PackageJsonManager;
use std::path::PathBuf;
use std::sync::Arc;
use yaml_rust::{yaml::Hash as YamlHash, Yaml, YamlLoader};

use error::AnyhowWrap;
pub use package::{Package, PackageError};
pub use tsconfig::*;

use crate::cli::CliOptions;

#[derive(Debug)]
pub struct Workspace {
    pub package_json: PackageJsonManager,
    pub root_dir: PathBuf,
    pub tsconfig: Option<Arc<TsConfig>>,
    pub pnpm_workspace: Option<YamlHash>,
}
impl From<Workspace> for Package {
    fn from(workspace: Workspace) -> Self {
        Self {
            package_json: workspace.package_json,
            root_dir: workspace.root_dir,
            tsconfig: workspace.tsconfig,
            parent: None,
        }
    }
}
impl PackageInterface for Workspace {
    fn package_json(&self) -> &PackageJson {
        self.package_json.as_ref()
    }
    fn tsconfig(&self) -> Option<&TsConfig> {
        self.tsconfig.as_ref().map(Arc::as_ref)
    }
    fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }
    fn parent(&self) -> Option<&dyn PackageInterface> {
        None
    }
}
impl Workspace {
    pub fn load(tsconfigs: &mut TsConfigStore, options: &CliOptions) -> Result<Self> {
        let mut manager = PackageJsonManager::new();
        let package_json = manager.locate_closest().map_err(AnyhowWrap::from)?;
        assert!(package_json.ends_with("package.json"));
        let _pkg = manager.read_ref().map_err(AnyhowWrap::from)?;
        let root_dir = package_json.parent().unwrap();
        let tsconfig = match &options.tsconfig {
            Some(tsconfig_path) => Some(tsconfigs.load(tsconfig_path.clone())?),
            None => None,
        };
        let mut ws = Self {
            package_json: manager,
            root_dir: root_dir.to_path_buf(),
            tsconfig,
            pnpm_workspace: None,
        };
        let entries = root_dir.read_dir().into_diagnostic()?;
        let mut found_tsconfig = false;
        let mut found_pnpm_workspace = false;
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let name = entry.file_name();
            if !name.is_ascii() {
                continue;
            }
            let name = name.to_str().unwrap(); // wont panic, we know its ascii
            match name {
                "tsconfig.json" if ws.tsconfig.is_none() => {
                    ws.tsconfig = Some(tsconfigs.load(root_dir.join(entry.path()))?);
                    found_tsconfig = true;
                    if found_pnpm_workspace {
                        break;
                    }
                }
                "pnpm-workspace.yaml" | "pnpm-workspace.yml" => {
                    let pnpm_workspace = std::fs::read_to_string(entry.path()).into_diagnostic()?;
                    let pnpm_workspace =
                        YamlLoader::load_from_str(&pnpm_workspace).into_diagnostic()?;
                    let Some(pnpm_workspace) =
                        pnpm_workspace
                            .into_iter()
                            .next()
                            .and_then(|yaml| match yaml {
                                Yaml::Hash(hash) => Some(hash),
                                _ => None,
                            })
                    else {
                        return Err(Report::msg("pnpm-workspace.yaml is missing a root object"));
                    };
                    ws.pnpm_workspace = Some(pnpm_workspace);
                    found_pnpm_workspace = true;
                    if found_tsconfig {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(ws)
    }

    pub fn workspace_globs(&self) -> Option<WorkspaceCollector> {
        if let Some(pnpm) = &self.pnpm_workspace {
            let key = Yaml::String("packages".into());
            return pnpm.get(&key).and_then(Yaml::as_vec).map(|packages| {
                packages
                    .iter()
                    .filter_map(Yaml::as_str)
                    .collect::<WorkspaceCollector>()
            });
        }

        self.package_json
            .as_ref()
            .workspaces
            .as_ref()
            .map(|ws| ws.iter().collect())
    }
}

#[derive(Debug)]
pub(crate) struct WorkspaceCollector {
    pub errors: Vec<PatternError>,
    pub patterns: Vec<Pattern>,
}
impl<A: AsRef<str>> FromIterator<A> for WorkspaceCollector {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let mut patterns: Vec<Pattern> = Vec::with_capacity(hint.1.unwrap_or(0));
        let mut errors: Vec<PatternError> = vec![];

        for pattern_str in iter {
            match Pattern::new(pattern_str.as_ref()) {
                Ok(pattern) => patterns.push(pattern),
                Err(e) => errors.push(e),
            }
        }

        Self { patterns, errors }
    }
}
