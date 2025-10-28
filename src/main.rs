use std::process::exit;

use cedar::{
    cli::{args::Args, commands::clean, error::CliError},
    core::error::BuilderError,
};

fn main() {
    let args = match Args::get() {
        Ok(a) => a,
        Err(e) => {
            print!("{e}");
            if let CliError::BuilderError(BuilderError::CompileError((_, p))) = e {
                println!("Unexpected failure! Cleaning up.");
                clean(p.parent().unwrap()).unwrap();
                exit(1);
            } else {
                println!("Unexpected failure!");
                exit(1);
            }
        }
    };
    match args.exec() {
        Ok(_) => (),
        Err(e) => eprintln!("{e}"),
    }
}
