use std::error::Error;

use cedar::cli::Args;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::get()?;
    args.exec()?;
    Ok(())
}
