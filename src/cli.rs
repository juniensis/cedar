use std::{env, error::Error, fmt::Display, fs, path::PathBuf};

use crate::structure::init::init;

#[derive(Debug)]
pub enum CliError {
    InvalidCommand,
    MissingArgument(&'static str),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::InvalidCommand => {
                write!(f, "Error: Invalid command was given.")
            }
            CliError::MissingArgument(arg) => {
                writeln!(f, "Error: Missing argument {}", arg)
            }
        }
    }
}

impl Error for CliError {}

pub struct Args {
    pub command: Commands,
    pub path: Option<PathBuf>,
}

pub enum Commands {
    Init,
    New,
    Build,
    Run,
    Help,
}

impl Args {
    pub fn get() -> Result<Self, CliError> {
        let mut cli = Self {
            command: Commands::Help,
            path: None,
        };

        let mut args = env::args().skip(1).enumerate();

        while let Some((i, arg)) = args.next() {
            match (i, arg.trim()) {
                (0, "init") => cli.command = Commands::Init,
                (0, "new") => {
                    let name = args.next();

                    if let Some((_, name)) = name {
                        cli.path = Some(
                            env::current_dir()
                                .expect("Error: Invalid current directory.")
                                .join(name),
                        );
                        cli.command = Commands::New
                    } else {
                        return Err(CliError::MissingArgument("name after command new."));
                    }
                }
                (0, "build") => cli.command = Commands::Build,
                (0, "run") => cli.command = Commands::Run,
                (0, "help") => cli.command = Commands::Help,
                (_, _) => {
                    return Err(CliError::InvalidCommand);
                }
            }
        }

        Ok(cli)
    }
    pub fn exec(&self) -> Result<(), Box<dyn Error>> {
        match self.command {
            Commands::Init => {
                let cwd = env::current_dir()?;

                println!("Initializing new project in the current directory.");
                println!("\t-> Generating directories and manifest.");

                init(&cwd)?;

                println!("\t-> Initializing git.");

                println!("\t-> Hashing manifest.");

                let manifest_bytes = fs::read(cwd.join("cedar.toml"))?;
                let crc = crc32fast::hash(&manifest_bytes);

                fs::write(cwd.join(".cedar/crc"), crc.to_le_bytes())?;

                println!("Done!");
                Ok(())
            }
            Commands::New => {
                println!("Initializing new project in the given directory.");
                println!("\t-> Generating directories and manifest.");

                let path = self.path.clone().unwrap();

                init(&path)?;

                println!("\t-> Initializing git.");

                println!("\t-> Hashing manifest.");

                let manifest_bytes = fs::read(path.join("cedar.toml"))?;
                let crc = crc32fast::hash(&manifest_bytes);

                fs::write(path.join(".cedar/crc"), crc.to_le_bytes())?;

                println!("Done!");

                Ok(())
            }
            Commands::Build => todo!(),
            Commands::Run => todo!(),
            Commands::Help => todo!(),
        }
    }
}
