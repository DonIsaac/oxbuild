mod error;
mod root;

use std::{env, num::NonZeroUsize, path::PathBuf};

use clap::{self, command, Arg, ArgMatches, ValueHint};
use miette::{Context, IntoDiagnostic, Result};

pub(crate) use root::Root;

pub fn cli() -> ArgMatches {
    command!()
        .arg(
            Arg::new("root")
                .value_hint(ValueHint::DirPath)
                .value_parser(path_parser)
                .help("Path to the root directory of your project")
                .long_help("Path to the root directory of your project.

By default, oxbuild will look for the nearest package.json starting at your CWD and walking up each parent. If you explicitly provide a path it will be used as-is and Oxbuild will not look for a package.json.")
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_hint(ValueHint::FilePath)
                .value_parser(path_parser)
                .help("Path to .oxbuildrc. Not yet supported"),
        )
        .arg(
            Arg::new("tsconfig")
                .short('p') // same as tsc
                .long("tsconfig")
                .value_hint(ValueHint::FilePath)
                .value_parser(path_parser)
                .help("Path to tsconfig.json")
                .long_help("Path to tsconfig.json.

By default, Oxbuild will look for a tsconfig.json next to the nearest package.json file. A tsconfig is not necessary; Oxbuild will assume your project does not use TypeScript.")
        )
        .arg(
            Arg::new("num_threads")
                .short('t')
                .long("threads")
                .value_parser(clap::value_parser!(NonZeroUsize))
                .help("Number of threads to use")
                .long_help("Number of threads to use. Defaults to the number of logical cores available on the system."),
        )
        .get_matches()
}

fn path_parser(v: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(v))
}

#[non_exhaustive]
pub struct CliOptions {
    pub root: Root,
    pub config: Option<PathBuf>,
    pub tsconfig: Option<PathBuf>,
    pub num_threads: NonZeroUsize,
}

impl CliOptions {
    pub fn new(matches: ArgMatches) -> Result<Self> {
        let project_dir = matches.get_one::<String>("root").map(PathBuf::from);
        let root = if let Some(project_dir) = project_dir {
            let project_dir = if project_dir
                .file_name()
                .is_some_and(|name| name.to_string_lossy() == "package.json")
            {
                project_dir.parent().unwrap().to_path_buf()
            } else {
                project_dir.clone()
            };
            Root::new_explicit(project_dir)?
        } else {
            Root::new_inferred()?
        };
        debug!("Root directory: '{}'", root.display());

        let config = root.resolve_file(
            matches.get_one::<PathBuf>("config"),
            ["oxbuild.json", ".oxbuild.json", ".oxbuildrc"],
        )?;

        let tsconfig =
            root.resolve_file(matches.get_one::<PathBuf>("tsconfig"), ["tsconfig.json"])?;

        let num_threads = match matches.get_one::<NonZeroUsize>("num_threads") {
            Some(n) => *n,
            None => {
                std::thread::available_parallelism().into_diagnostic().with_context(|| "Failed to determine number of threads available. Please provide this explicitly using -t,--threads <n>")?
            }
        };

        Ok(Self {
            root,
            config,
            tsconfig,
            num_threads,
        })
    }
}
