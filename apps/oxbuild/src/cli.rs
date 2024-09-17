use std::{
    env,
    fs::{self, DirEntry},
    path::PathBuf,
};

use clap::{command, Arg, ArgMatches, ValueHint};
use miette::{IntoDiagnostic, Report, Result, WrapErr};

pub fn cli() -> ArgMatches {
    command!()
        // .arg(
        //     Arg::new("input")
        //         .value_hint(ValueHint::DirPath)
        //         .help("Directory containing your source code. Defaults to CWD"),
        // )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_hint(ValueHint::FilePath)
                .help("Path to .oxbuildrc"),
        )
        .arg(
            Arg::new("tsconfig")
                .short('t')
                .long("tsconfig")
                .value_hint(ValueHint::FilePath)
                .help("Path to tsconfig.json"),
        )
        .arg(
            Arg::new("cwd")
                .long("cwd")
                .value_hint(ValueHint::DirPath)
                .help("Root directory for the project. Defaults to the current working directory."),
        )
        .get_matches()
}

pub struct CliOptions {
    root: Root,
    pub config: Option<PathBuf>,
    pub tsconfig: Option<PathBuf>,
    pub input: PathBuf,
    pub output: PathBuf,
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
            input: "./src".into(),
            output: "./dist".into(),
        })
    }

    pub fn cwd(&self) -> &PathBuf {
        self.root.cwd()
    }
}

struct Root {
    cwd: PathBuf,
    stat: Vec<DirEntry>,
}

impl Root {
    pub fn new() -> Result<Self> {
        let cwd = env::current_dir().into_diagnostic()?;
        assert!(cwd.is_dir());
        let stat = fs::read_dir(&cwd)
            .into_diagnostic()
            .context("Failed to read files in cwd")?;
        let stat: Vec<_> = stat.flatten().collect();

        Ok(Self { cwd, stat })
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    pub fn resolve_file<I>(
        &self,
        path: Option<&PathBuf>,
        possible_names: I,
    ) -> Result<Option<PathBuf>>
    where
        I: IntoIterator<Item = &'static str>,
    {
        if let Some(path) = path {
            if !path.exists() {
                return Err(Report::msg(format!("File not found: {}", path.display())));
            }
            if path.is_dir() {
                return Err(Report::msg(format!(
                    "{} is a directory, not a file",
                    path.display()
                )));
            }

            return self
                .cwd
                .join(path)
                .canonicalize()
                .into_diagnostic()
                .map(Some);
        }

        Ok(self.find(possible_names))
    }

    pub fn find<I>(&self, names: I) -> Option<PathBuf>
    where
        I: IntoIterator<Item = &'static str>,
    {
        for name in names.into_iter() {
            let search_result = self
                .stat
                .iter()
                .filter(|e| e.file_type().map_or(true, |ft| ft.is_file()))
                .find(|entry| entry.file_name().to_str().unwrap() == name);

            if let Some(entry) = search_result {
                return Some(entry.path());
            }
        }

        None
    }
}
