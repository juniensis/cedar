use crate::commands::{build, help, init, new, run};
use crate::error::CliError;
use std::{env, error::Error, path::PathBuf};

/// A structure for holding the command line arguments.
///
/// # Fields
///
/// * 'command' - An instance of the Command enum representing what part of the
///         program to execute.
/// * 'path' - An optional PathBuf pointing to the project directory. It is
///         optional because only the new command requires a path, the rest
///         work in the current working directory.
///
#[derive(Clone)]
pub struct Args {
    pub command: Commands,
    pub path: PathBuf,
    pub flags: Vec<Flags>,
}

/// An enum for holding the possible commands.
///
/// # Members
///
/// * 'Init' -  Initializes a project in the current directory.
/// * 'New' - Intializes a project in the given relative or absolute path.
/// * 'Build' - Compiles and links all the fiels in src and include.
/// * 'Run' - Compiles/links and runs the program.
/// * 'Help' - Displays the help message.
///
#[derive(Clone, Copy)]
pub enum Commands {
    Init,
    New,
    Build,
    Run,
    Help,
}

/// An enum for holding possible flags.
///
/// # Members
///
/// * 'Git' - Initalizes a git repositiory in the project.
///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flags {
    Git,
}

impl Args {
    // Gets the environment arguments and returns an Args struct with them.
    pub fn get() -> Result<Self, CliError> {
        let mut cli = Self {
            command: Commands::Help,
            path: env::current_dir().unwrap(),
            flags: Vec::new(),
        };

        let mut args = env::args().skip(1).enumerate();

        while let Some((i, arg)) = args.next() {
            match (i, arg.trim()) {
                (0, "init") => cli.command = Commands::Init,
                (0, "new") => {
                    let name = args.next();

                    if let Some((_, name)) = name {
                        cli.path = env::current_dir()
                            .expect("Error: Invalid current directory.")
                            .join(name.trim_start_matches("/"));

                        cli.command = Commands::New
                    } else {
                        return Err(CliError::MissingArgument("name after command new."));
                    }
                }
                (0, "build") => cli.command = Commands::Build,
                (0, "run") => cli.command = Commands::Run,
                (0, "help") => cli.command = Commands::Help,
                (0, _) => {
                    return Err(CliError::InvalidCommand);
                }
                (_, "--git") | (_, "-g") => {
                    cli.flags.push(Flags::Git);
                }
                (_, _) => {}
            }
        }

        Ok(cli)
    }
    pub fn exec(&self) -> Result<(), Box<dyn Error>> {
        match self.command {
            Commands::Init => {
                init(&self.path, self.flags.contains(&Flags::Git))?;
                Ok(())
            }
            Commands::New => {
                new(&self.path, self.flags.contains(&Flags::Git))?;
                Ok(())
            }
            Commands::Build => {
                build(&self.path)?;
                Ok(())
            }
            Commands::Run => {
                run(&self.path)?;
                Ok(())
            }
            Commands::Help => {
                help();
                Ok(())
            }
        }
    }
}
