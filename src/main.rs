mod cli;
mod options;
mod walk;
mod compiler;

use miette::Result;
use std::time::Instant;

use cli::{cli, CliOptions};
use options::OxbuildOptions;

fn main() -> Result<()> {
    let matches = cli();
    let opts = CliOptions::new(matches).and_then(OxbuildOptions::new)?;

    let start = Instant::now();
    let mut walker = walk::WalkerBuilder::new(opts);
    walker.walk(10); // TODO: configure based on threads available
    let duration = start.elapsed();

    println!("Finished in {:2}ms", duration.as_millis());

    Ok(())
}
