use std::{env, error::Error, fmt::Display};

#[derive(Debug)]
pub enum CliError {
    InvalidCommand,
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::InvalidCommand => {
                write!(f, "Error: Invalid command was given.")
            }
        }
    }
}

impl Error for CliError {}

pub struct Args {
    pub command: Commands,
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
        };

        let args = env::args().skip(1).enumerate();

        for (i, arg) in args {
            match (i, arg.trim()) {
                (0, "init") => cli.command = Commands::Init,
                (0, "new") => cli.command = Commands::New,
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
    pub fn exec(&self) {
        match self.command {
            Commands::Init => todo!(),
            Commands::New => todo!(),
            Commands::Build => todo!(),
            Commands::Run => todo!(),
            Commands::Help => todo!(),
        }
    }
}
