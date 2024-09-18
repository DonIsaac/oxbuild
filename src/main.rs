mod cli;
mod compiler;
mod options;
mod walk;

use miette::Result;
use std::time::Instant;

use cli::{cli, CliOptions};
use options::OxbuildOptions;

#[allow(clippy::print_stdout)]
fn main() -> Result<()> {
    let matches = cli();
    let opts = CliOptions::new(matches).and_then(OxbuildOptions::new)?;
    let num_threads = opts.num_threads.get();

    let start = Instant::now();
    let mut walker = walk::WalkerBuilder::new(opts);
    walker.walk(num_threads);
    let duration = start.elapsed();

    println!(
        "Finished in {:2}ms using {num_threads} threads.",
        duration.as_millis()
    );

    Ok(())
}
