use std::process::exit;

use cedar::cli::{args::Args, error::CliError};

fn main() -> Result<(), CliError> {
    let args = match Args::get() {
        Ok(a) => a,
        Err(e) => {
            print!("{e}");
            exit(1);
        }
    };
    args.exec()
}
