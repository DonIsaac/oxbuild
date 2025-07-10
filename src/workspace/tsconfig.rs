use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use miette::{IntoDiagnostic, Result};
use serde::Deserialize;

#[derive(Debug, Default)]
enum LoadState<T> {
    Loaded(T),
    #[default]
    Pending,
}
impl<T> LoadState<T> {
    fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Loaded(t) => Some(t),
            Self::Pending => None,
        }
    }
}
#[derive(Default, Debug)]
pub struct TsConfigStore {
    configs: HashMap<PathBuf, LoadState<Arc<TsConfig>>>,
}
impl TsConfigStore {
    pub fn load(&mut self, tsconfig_path: PathBuf) -> Result<Arc<TsConfig>> {
        assert!(tsconfig_path.is_absolute());
        debug!("loading tsconfig at '{}'", tsconfig_path.display());
        {
            if let Some(existing) = self.configs.get(&tsconfig_path) {
                return existing.as_ref().map(Arc::clone).ok_or_else(|| {
                    miette::miette!("Circular dependency detected: {}", tsconfig_path.display())
                });
            }
        }
        let tsconfig = TsConfig::from_file(&tsconfig_path)?;
        if let Some(extends) = tsconfig.extends.as_ref() {
            self.configs
                .insert(tsconfig_path.clone(), LoadState::Pending);
            let full_path = tsconfig_path
                .parent()
                .unwrap()
                .join(extends)
                .canonicalize()
                .into_diagnostic()?;
            let parent = self.load(full_path)?;
            let tsconfig = parent.merge(tsconfig);
            let prev = self
                .configs
                .insert(tsconfig_path.clone(), LoadState::Loaded(Arc::new(tsconfig)));
            assert!(matches!(prev, Some(LoadState::Pending)));
            Ok(Arc::clone(self.configs[&tsconfig_path].as_ref().unwrap()))
        } else {
            let prev = self
                .configs
                .insert(tsconfig_path.clone(), LoadState::Loaded(Arc::new(tsconfig)));
            assert!(prev.is_none());
            Ok(Arc::clone(self.configs[&tsconfig_path].as_ref().unwrap()))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    pub extends: Option<PathBuf>,
    // TODO: tsconfig extends
    pub compiler_options: Option<TsConfigCompilerOptions>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TsConfigCompilerOptions {
    // TODO: parse more fields as needed
    pub root_dir: Option<PathBuf>,
    /// Specify an output folder for all emitted files.
    pub out_dir: Option<PathBuf>,
    /// Set the JavaScript language version for emitted JavaScript and include compatible library declarations.
    pub target: Option<String>,
    /// Specify what module code is generated.
    pub module: Option<String>,
    /// Specify what JSX code is generated.
    /// <https://www.typescriptlang.org/tsconfig/#jsx>
    pub jsx: Option<TsConfigJsx>,
    /// Specify the JSX factory function used when targeting React JSX emit, e.g. 'createElement' or 'h'.
    /// <https://www.typescriptlang.org/tsconfig/#jsxFactory>
    pub jsx_factory: Option<String>,
    /// Specify the JSX fragment factory function to use when targeting react
    /// JSX emit with jsxFactory compiler option is specified, e.g. Fragment.
    /// <https://www.typescriptlang.org/tsconfig/#jsxFragmentFactory>
    pub jsx_fragment_factory: Option<String>,
    /// Enable experimental support for legacy experimental decorators.
    pub experimental_decorators: Option<bool>,
    /// Emit design-type metadata for decorated declarations in source files.
    pub emit_decorator_metadata: Option<bool>,
    /// Create source map files for emitted JavaScript files.
    pub source_map: Option<bool>,
    /// Include sourcemap files inside the emitted JavaScript.
    pub inline_source_map: Option<bool>,
    /// Allow JavaScript files to be a part of your program.
    pub allow_js: Option<bool>,
    /// Generate .d.ts files from TypeScript and JavaScript files in your project.
    pub declaration: Option<bool>,
    /// Generate sourcemaps for d.ts files.
    pub declaration_map: Option<bool>,
    /// Only output d.ts files and not JavaScript files.
    pub emit_declaration_only: Option<bool>,
    ///  Disable emitting declarations that have '@internal' in their JSDoc comments.
    pub strip_internal: Option<bool>,
    /// Require sufficient annotation on exports so other tools can trivially generate declaration files.
    pub isolated_declarations: Option<bool>,
    /// Specify the base directory to resolve non-relative module names.
    pub base_url: Option<String>,
    /// Specify a set of entries that re-map imports to additional lookup locations.
    pub paths: Option<HashMap<String, Vec<String>>>,
}

/// <https://www.typescriptlang.org/tsconfig/#jsx>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TsConfigJsx {
    #[serde(rename = "react-jsx")]
    ReactJsx,
    #[serde(rename = "react-jsxdev")]
    ReactJsxDev,
    #[serde(rename = "preserve")]
    Preserve,
    #[serde(rename = "react-native")]
    ReactNative,
    #[serde(rename = "react")]
    React,
}
impl TsConfigJsx {
    pub fn is_dev(self) -> bool {
        matches!(self, Self::ReactJsxDev)
    }
}

impl TsConfig {
    pub fn parse(mut source_text: String) -> Result<Self> {
        json_strip_comments::strip(&mut source_text).unwrap();

        let mut tsconfig: Self = serde_json::from_str(&source_text).into_diagnostic()?;
        if tsconfig.exclude.as_ref().is_some_and(|ex| ex.is_empty()) {
            tsconfig.exclude = None;
        }
        Ok(tsconfig)
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let source_text = std::fs::read_to_string(path).into_diagnostic()?;
        Self::parse(source_text)
    }
}

impl Merge for TsConfig {
    fn merge(&self, other: Self) -> Self {
        let compiler_options = match (self.compiler_options.as_ref(), other.compiler_options) {
            (Some(a), Some(b)) => Some(a.merge(b)),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        Self {
            extends: self.extends.clone(),
            compiler_options,
            exclude: other.exclude.or_else(|| self.exclude.clone()),
        }
    }
}
impl Merge for TsConfigCompilerOptions {
    fn merge(&self, other: Self) -> Self {
        Self {
            root_dir: self.root_dir.merge(other.root_dir),
            out_dir: self.out_dir.merge(other.out_dir),
            target: self.target.merge(other.target),
            module: self.module.merge(other.module),
            jsx: self.jsx.merge(other.jsx),
            jsx_factory: self.jsx_factory.merge(other.jsx_factory),
            jsx_fragment_factory: self.jsx_fragment_factory.merge(other.jsx_fragment_factory),
            experimental_decorators: self
                .experimental_decorators
                .merge(other.experimental_decorators),
            emit_decorator_metadata: self
                .emit_decorator_metadata
                .merge(other.emit_decorator_metadata),
            source_map: self.source_map.merge(other.source_map),
            inline_source_map: self.inline_source_map.merge(other.inline_source_map),
            allow_js: self.allow_js.merge(other.allow_js),
            declaration: self.declaration.merge(other.declaration),
            declaration_map: self.declaration_map.merge(other.declaration_map),
            emit_declaration_only: self
                .emit_declaration_only
                .merge(other.emit_declaration_only),
            strip_internal: self.strip_internal.merge(other.strip_internal),
            isolated_declarations: self
                .isolated_declarations
                .merge(other.isolated_declarations),
            base_url: self.base_url.merge(other.base_url),
            paths: self.paths.merge(other.paths),
        }
    }
}

trait Merge {
    /// Apply `other` on top of `this`. other's properties take precedence over `this`'s.
    fn merge(&self, other: Self) -> Self;
}

impl<T: Clone> Merge for Option<T> {
    fn merge(&self, other: Self) -> Self {
        self.clone().or(other)
    }
}
