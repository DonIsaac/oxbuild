use std::{collections::HashMap, path::PathBuf};

use miette::{IntoDiagnostic, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    // TODO: tsconfig extends
    pub compiler_options: Option<TsConfigCompilerOptions>,
}
impl TsConfig {
    pub fn compiler_options(&self) -> Option<&TsConfigCompilerOptions> {
        self.compiler_options.as_ref()
    }
}

#[derive(Debug, Default, Deserialize)]
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

        serde_json::from_str(&source_text).into_diagnostic()
    }
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let source_text = std::fs::read_to_string(path).into_diagnostic()?;
        Self::parse(source_text)
    }
}
