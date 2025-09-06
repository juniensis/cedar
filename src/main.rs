use std::process::exit;

use cedar::{
    cli::{args::Args, commands::clean, error::CliError},
    core::error::BuilderError,
};

fn main() -> Result<(), CliError> {
    let args = match Args::get() {
        Ok(a) => a,
        Err(e) => {
            print!("{e}");
            if let CliError::BuilderError(BuilderError::CompileError((_, p))) = e {
                println!("CLEANING");
                clean(p.parent().unwrap()).unwrap();
                exit(1);
            } else {
                println!("NOT");
                exit(1);
            }
        }
    };
    args.exec()
}
