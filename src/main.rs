mod cli;
mod compiler;
mod options;
mod reporter;
mod walk;

use std::{
    io::{BufWriter, Write},
    process::ExitCode,
    thread,
    time::Instant,
};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use miette::{IntoDiagnostic, Result};

use crate::{
    cli::{cli, CliOptions},
    options::OxbuildOptions,
    reporter::Reporter,
};

#[allow(clippy::print_stdout)]
fn main() -> Result<ExitCode> {
    pretty_env_logger::init();
    let matches = cli();
    let opts = CliOptions::new(matches).and_then(OxbuildOptions::new)?;
    let num_threads = opts.num_threads.get();

    let (mut reporter, report_sender) = Reporter::new();

    let start = Instant::now();

    let handle = thread::spawn(move || {
        let mut walker = walk::WalkerBuilder::new(opts, report_sender);
        walker.walk(num_threads);
    });

    let mut stderr = BufWriter::new(std::io::stderr());
    let result = reporter.run(&mut stderr);
    stderr.flush().into_diagnostic()?;
    handle.join().unwrap();

    let duration = start.elapsed();
    let num_errors = result.errors_count();
    let num_warnings = result.warnings_count();
    let did_fail = num_errors > 0;

    if num_errors > 0 && num_warnings > 0 {
        println!(
            "Finished in {:2}ms with {num_errors} errors and {num_warnings} warnings using {num_threads} threads.",
            duration.as_millis()
        );
    } else {
        println!(
            "Finished in {:2}ms using {num_threads} threads.",
            duration.as_millis()
        );
    }

    Ok(ExitCode::from(u8::from(did_fail)))
}
