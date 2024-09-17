mod cli;
mod walk;

use miette::Result;
use std::time::Instant;

use cli::{cli, CliOptions};

fn main() -> Result<()> {
    let matches = cli();
    let opts = CliOptions::new(matches)?;

    let start = Instant::now();
    let mut walker = walk::WalkerBuilder::new(opts);
    walker.walk(10); // TODO: configure based on threads available
    let duration = start.elapsed();

    println!("Finished in {:2}ms", duration.as_millis());

    Ok(())
}
