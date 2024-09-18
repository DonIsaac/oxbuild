mod error;
mod root;

use std::{env, num::NonZeroUsize, path::PathBuf};

use clap::{self, command, Arg, ArgMatches, ValueHint};
use miette::{Context, IntoDiagnostic, Result};

pub(crate) use root::Root;

pub fn cli() -> ArgMatches {
    command!()
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_hint(ValueHint::FilePath)
                .help("Path to .oxbuildrc. Not yet supported"),
        )
        .arg(
            Arg::new("tsconfig")
                .short('p') // same as tsc
                .long("tsconfig")
                .value_hint(ValueHint::FilePath)
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

#[non_exhaustive]
pub struct CliOptions {
    pub root: Root,
    pub config: Option<PathBuf>,
    pub tsconfig: Option<PathBuf>,
    pub num_threads: NonZeroUsize,
}

impl CliOptions {
    pub fn new(matches: ArgMatches) -> Result<Self> {
        let root = Root::new()?;

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
