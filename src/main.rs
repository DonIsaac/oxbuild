mod cli;
mod compiler;
mod options;
mod reporter;
mod walk;

use std::{process::ExitCode, thread, time::Instant};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use miette::Result;

use crate::{
    cli::{cli, CliOptions},
    options::OxbuildOptions,
    reporter::{DiagnosticSender, Reporter},
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
        let mut walker = walk::WalkerBuilder::new(opts, report_sender.clone());
        walker.walk(num_threads);
        report_sender.send(None).unwrap();
    });

    reporter.run();
    handle.join().unwrap();

    let duration = start.elapsed();
    let num_errors = reporter.errors_count();
    let num_warnings = reporter.warnings_count();
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
