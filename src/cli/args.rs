use std::{env, fs, path::PathBuf};

use crate::cli::{
    commands::{build, clean, help, init, new, run},
    error::CliError,
};

#[derive(Debug)]
pub struct Args {
    pub command: Command,
    pub path: PathBuf,
    pub flags: Vec<Flag>,
}

#[derive(Debug)]
pub enum Command {
    Init,
    New,
    Build,
    Run,
    Help,
    Clean,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Flag {
    Git,
}

impl Args {
    pub fn get() -> Result<Self, CliError> {
        let mut cli = Self {
            command: Command::Help,
            path: env::current_dir()?,
            flags: Vec::new(),
        };

        let mut args = env::args().skip(1).enumerate();

        while let Some((i, arg)) = args.next() {
            match (i, arg.trim()) {
                (0, "init") => cli.command = Command::Init,
                (0, "new") => {
                    let name = args.next();

                    if let Some((_, name)) = name {
                        fs::create_dir_all(&name)?;
                        cli.path = fs::canonicalize(&name)?;
                        cli.command = Command::New;
                    } else {
                        return Err(CliError::MissingArgument(
                            "no name following the command 'new'.",
                        ));
                    }
                }
                (0, "build") => cli.command = Command::Build,
                (0, "run") => cli.command = Command::Run,
                (0, "help") => cli.command = Command::Help,
                (0, "clean") => cli.command = Command::Clean,
                (_, "--git" | "-g") => cli.flags.push(Flag::Git),
                (0, _) => {
                    help();
                    return Err(CliError::InvalidCommand);
                }
                _ => {}
            }
        }

        Ok(cli)
    }
    pub fn exec(&self) -> Result<(), CliError> {
        let git = self.flags.contains(&Flag::Git);
        match self.command {
            Command::Init => init(&self.path, git)?,
            Command::New => new(&self.path, git)?,
            Command::Build => build(&self.path)?,
            Command::Run => run(&self.path)?,
            Command::Clean => clean(&self.path)?,
            Command::Help => help(),
        }
        Ok(())
    }
}
