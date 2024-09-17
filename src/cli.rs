mod error;
mod root;

use std::{env, path::PathBuf};

use clap::{command, Arg, ArgMatches, ValueHint};
use miette::Result;

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
                .help("Path to tsconfig.json"),
        )
        .get_matches()
}

pub struct CliOptions {
    pub root: Root,
    pub config: Option<PathBuf>,
    pub tsconfig: Option<PathBuf>,
    // pub input: PathBuf,
    // pub output: PathBuf,
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

        Ok(Self {
            root,
            config,
            tsconfig,
        })
    }
}
