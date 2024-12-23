use std::error::Error;

use crate::cli::Args;

mod cli;
mod commands;
mod error;
mod manifest;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::get()?;
    args.exec()?;
    Ok(())
}
