use std::{
    fmt,
    fs::{self},
    num::NonZeroUsize,
    path::PathBuf,
};

use miette::{IntoDiagnostic, Report, Result, WrapErr};
use oxc::transformer::TransformOptions;
// use package_json::{PackageJson, PackageJsonManager};
use serde::Deserialize;

use crate::cli::{CliOptions, Root};

// use crate::error::AnyError;

pub struct OxbuildOptions {
    pub root: Root,
    /// Emit `.d.ts` files using `isolatedModules` option.
    pub isolated_declarations: bool,
    /// Path to the folder containing source files to compile.
    pub src: PathBuf,
    /// Path to output folder where compiled code will be written.
    pub dist: PathBuf,
    pub num_threads: NonZeroUsize,
    pub transform_options: TransformOptions,
}

impl OxbuildOptions {
    pub fn new(cli: CliOptions) -> Result<Self> {
        let CliOptions {
            root,
            config: _config,
            tsconfig,
            num_threads,
        } = cli;

        let tsconfig = root
            .resolve_file(tsconfig.as_ref(), ["tsconfig.json"])?
            .map(|tsconfig_path| {
                fs::read_to_string(&tsconfig_path)
                    .into_diagnostic()
                    .with_context(|| {
                        format!("Failed to read TSConfig at {}", tsconfig_path.display())
                    })
                    .and_then(TsConfig::parse)
            })
            .transpose()?;

        // TODO: config files
        // let config = root.resolve_file(
        //     config.as_ref(),
        //     ["oxbuild.json", ".oxbuild.json", ".oxbuildrc"],
        // )?;

        let co = tsconfig.as_ref().and_then(TsConfig::compiler_options);
        let src = if let Some(root_dir) = co.and_then(|co| co.root_dir.as_ref()) {
            root.resolve(root_dir)
        } else {
            let src = root.join("src").to_path_buf();
            if !src.exists() {
                return Err(Report::msg("src directory does not exist. Please explicitly provide a path to your source files.".to_string()));
            }
            src
        };
        if !src.is_dir() {
            return Err(Report::msg(format!(
                "rootDir in tsconfig.json is not a directory: {}",
                src.display()
            )));
        }

        let dist = if let Some(out_dir) = co.and_then(|co| co.out_dir.as_ref()) {
            root.resolve(out_dir)
        } else {
            root.join("dist").to_path_buf()
        };

        // TODO: clean dist dir?
        if !dist.exists() {
            fs::create_dir(&dist).into_diagnostic()?;
        }
        if !dist.is_dir() {
            return Err(Report::msg(format!(
                "Invalid output directory: '{}' is not a directory",
                dist.display()
            )));
        }

        let isolated_declarations = co
            .and_then(|co| co.isolated_declarations)
            // no tsconfig means they're using JavaScript. We can't emit .d.ts files in that case.
            .unwrap_or(false);

        let mut transform_options = tsconfig
            .as_ref()
            .map(|tsconfig| tsconfig.transform_options())
            .transpose()?
            .unwrap_or_default();

        transform_options.cwd = root.to_path_buf();

        Ok(Self {
            root,
            isolated_declarations,
            src,
            dist,
            num_threads,
            transform_options,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TsConfig {
    // TODO: tsconfig extends
    compiler_options: Option<TsConfigCompilerOptions>,
}
impl TsConfig {
    fn compiler_options(&self) -> Option<&TsConfigCompilerOptions> {
        self.compiler_options.as_ref()
    }
}

/// [`compilerOptions`](https://www.typescriptlang.org/tsconfig/#compilerOptions) in a
/// `tsconfig.json` file.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsConfigCompilerOptions {
    // TODO: parse more fields as needed
    root_dir: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    isolated_declarations: Option<bool>,
    /// https://www.typescriptlang.org/tsconfig/#target
    #[serde(default)]
    target: TsConfigTarget,
}

/// https://www.typescriptlang.org/tsconfig/#target
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")] // just needed for lowercasing values
pub enum TsConfigTarget {
    /// Not supported by oxc
    ES3,
    ES5,
    /// Same as es2015
    ES6,
    /// Same as es6
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    #[default]
    ESNext,
}
impl fmt::Display for TsConfigTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ESNext => "ESNext".fmt(f),
            Self::ES2023 => "ES2023".fmt(f),
            Self::ES2022 => "ES2022".fmt(f),
            Self::ES2021 => "ES2021".fmt(f),
            Self::ES2020 => "ES2020".fmt(f),
            Self::ES2019 => "ES2019".fmt(f),
            Self::ES2018 => "ES2018".fmt(f),
            Self::ES2017 => "ES2017".fmt(f),
            Self::ES2016 => "ES2016".fmt(f),
            Self::ES2015 => "ES2015".fmt(f),
            Self::ES6 => "ES6".fmt(f),
            Self::ES5 => "ES5".fmt(f),
            Self::ES3 => "ES3".fmt(f),
        }
    }
}
impl TsConfigTarget {
    /// Returns [`true`] if this version of ECMAScript is not supported by `oxc_transform`
    fn is_unsupported(self) -> bool {
        matches!(self, Self::ES3)
    }
}

/// A parsed `tsconfig.json` file.
///
/// See: [TSConfig Reference](https://www.typescriptlang.org/tsconfig/)
impl TsConfig {
    pub fn parse(mut source_text: String) -> Result<Self> {
        json_strip_comments::strip(&mut source_text).unwrap();

        serde_json::from_str(&source_text).into_diagnostic()
    }

    pub fn transform_options(&self) -> Result<TransformOptions> {
        let co = self.compiler_options();
        let target = co.map(|co| co.target).unwrap_or_default();

        if target.is_unsupported() {
            return Err(Report::msg(format!(
                "Oxbuild does not support compiling to {target}. Please use a higher target version.",
            )));
        }
        let mut options = TransformOptions::default();

        // TODO: set presets once TransformOptions supports factories that take a target ECMAScript version
        if target <= TsConfigTarget::ES2021 {
            options.es2021.logical_assignment_operators = true
        }

        Ok(options)
    }
}
