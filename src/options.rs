use crate::cli::{CliOptions, Root};
use std::{
    fs::{self},
    path::PathBuf,
};

use miette::{IntoDiagnostic, Report, Result, WrapErr};
// use package_json::{PackageJson, PackageJsonManager};
use serde::Deserialize;

// use crate::error::AnyError;

pub struct OxbuildOptions {
    pub root: Root,
    /// Emit `.d.ts` files using `isolatedModules` option.
    pub isolated_declarations: bool,
    /// Path to the folder containing source files to compile.
    pub src: PathBuf,
    /// Path to output folder where compiled code will be written.
    pub dist: PathBuf,
    // package_json: PackageJson,
    // tsconfig: Option<PathBuf>, // TODO
}

impl OxbuildOptions {
    pub fn new(cli: CliOptions) -> Result<Self> {
        let CliOptions {
            root,
            config: _config,
            tsconfig,
        } = cli;

        let tsconfig = root
            .resolve_file(tsconfig.as_ref(), ["tsconfig.json"])?
            .map(|tsconfig_path| {
                fs::read_to_string(&tsconfig_path)
                    .into_diagnostic()
                    .with_context(|| {
                        format!("Failed to read TSConfig at {}", tsconfig_path.display())
                    })
                    .and_then(TsConfig::new)
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
                return Err(Report::msg("src directory does not exist. Please explicity provide a path to your source files.".to_string()));
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
            let dist = root.join("dist").to_path_buf();
            if !dist.exists() {
                fs::create_dir(&dist).into_diagnostic()?;
            }
            // TODO: clean dist dir?
            dist
        };
        assert!(dist.is_dir()); // FIXME: handle errors

        let isolated_declarations = co
            .and_then(|co| co.isolated_declarations)
            // no tsconfig means they're using JavaScript. We can't emit .d.ts files in that case.
            .unwrap_or(false);

        Ok(Self {
            root,
            isolated_declarations,
            src,
            dist,
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsConfigCompilerOptions {
    // TODO: parse more fields as needed
    root_dir: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    isolated_declarations: Option<bool>,
}

impl TsConfig {
    pub fn new(mut source_text: String) -> Result<Self> {
        json_strip_comments::strip(&mut source_text).unwrap();

        serde_json::from_str(&source_text).into_diagnostic()
    }
}
