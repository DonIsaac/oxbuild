// use crate::cli::{CliOptions, Root};
// use std::{
//     fs::{self},
//     num::NonZeroUsize,
//     path::PathBuf,
// };

// use log::{debug, trace};
// use miette::{IntoDiagnostic, Report, Result, WrapErr};
// // use package_json::{PackageJson, PackageJsonManager};
// use serde::Deserialize;

// #[derive(Debug)]
// pub struct OxbuildOptions {
//     pub root: Root,
//     /// Emit `.d.ts` files using `isolatedDeclarations` option.
//     ///
//     /// When [`Some`], declarations will be emitted using the provided options.
//     /// When [`None`], declaration emit is disabled.
//     pub isolated_declarations: Option<DeclarationsOptions>,
//     /// Path to the folder containing source files to compile.
//     pub src: PathBuf,
//     /// Path to output folder where compiled code will be written.
//     pub dist: PathBuf,
//     pub num_threads: NonZeroUsize,
//     // package_json: PackageJson,
//     // tsconfig: Option<PathBuf>, // TODO
// }

// #[derive(Debug, Clone)]
// pub struct DeclarationsOptions {
//     pub strip_internal: bool,
// }

// impl OxbuildOptions {
//     pub fn new(cli: CliOptions) -> Result<Self> {
//         let CliOptions {
//             root,
//             config: _config,
//             tsconfig,
//             num_threads,
//         } = cli;

//         let tsconfig = root
//             .resolve_file(tsconfig.as_ref(), ["tsconfig.json"])?
//             .map(|tsconfig_path| {
//                 debug!("Reading tsconfig at '{}'", tsconfig_path.display());
//                 fs::read_to_string(&tsconfig_path)
//                     .into_diagnostic()
//                     .with_context(|| {
//                         format!("Failed to read TSConfig at {}", tsconfig_path.display())
//                     })
//                     .and_then(TsConfig::parse)
//             })
//             .transpose()?;

//         // TODO: config files
//         // let config = root.resolve_file(
//         //     config.as_ref(),
//         //     ["oxbuild.json", ".oxbuild.json", ".oxbuildrc"],
//         // )?;

//         let co = tsconfig.as_ref().and_then(TsConfig::compiler_options);
//         let src = if let Some(root_dir) = co.and_then(|co| co.root_dir.as_ref()) {
//             debug!(
//                 "Resolving rootDir from tsconfig.json: '{}'",
//                 root_dir.display()
//             );
//             root.resolve(root_dir)
//         } else {
//             debug!("Using default src directory");
//             let src = root.join("src").to_path_buf();
//             if !src.exists() {
//                 return Err(Report::msg("src directory does not exist. Please explicitly provide a path to your source files.".to_string()));
//             }
//             src
//         };
//         if !src.is_dir() {
//             return Err(Report::msg(format!(
//                 "rootDir in tsconfig.json is not a directory: {}",
//                 src.display()
//             )));
//         }
//         trace!("src directory: '{}'", src.display());

//         let dist = if let Some(out_dir) = co.and_then(|co| co.out_dir.as_ref()) {
//             debug!(
//                 "Resolving outDir from tsconfig.json: '{}'",
//                 out_dir.display()
//             );
//             root.join(out_dir)
//         } else {
//             debug!("Using default dist directory");
//             root.join("dist").to_path_buf()
//         };
//         // TODO: clean dist dir?
//         if !dist.exists() {
//             trace!("Creating dist directory at '{}'", dist.display());
//             fs::create_dir_all(&dist).into_diagnostic()?;
//         }
//         assert!(dist.is_dir()); // FIXME: handle errors
//         let dist = dist
//             .canonicalize()
//             .into_diagnostic()
//             .wrap_err("Failed to canonicalize dist directory")?;
//         trace!("dist directory: '{}'", dist.display());

//         // no tsconfig means they're using JavaScript. We can't emit .d.ts files in that case.
//         let isolated_declarations = co.and_then(|co| {
//             co.isolated_declarations.unwrap_or(false).then(|| {
//                 debug!("Enabling .d.ts emit");
//                 DeclarationsOptions {
//                     strip_internal: co.strip_internal.unwrap_or(false),
//                 }
//             })
//         });

//         Ok(Self {
//             root,
//             isolated_declarations,
//             src,
//             dist,
//             num_threads,
//         })
//     }
// }

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct TsConfig {
//     // TODO: tsconfig extends
//     compiler_options: Option<TsConfigCompilerOptions>,
// }
// impl TsConfig {
//     fn compiler_options(&self) -> Option<&TsConfigCompilerOptions> {
//         self.compiler_options.as_ref()
//     }
// }

// #[derive(Debug, Default, Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct TsConfigCompilerOptions {
//     // TODO: parse more fields as needed
//     root_dir: Option<PathBuf>,
//     out_dir: Option<PathBuf>,
//     strip_internal: Option<bool>,
//     isolated_declarations: Option<bool>,
// }

// impl TsConfig {
//     pub fn parse(mut source_text: String) -> Result<Self> {
//         json_strip_comments::strip(&mut source_text).unwrap();

//         serde_json::from_str(&source_text).into_diagnostic()
//     }
// }
