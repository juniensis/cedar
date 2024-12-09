use crate::{
    build::build,
    structure::{init::init, manifest::Manifest},
};
use std::{env, error::Error, fmt::Display, fs, path::PathBuf, process};

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

#[derive(Clone)]
pub struct Args {
    pub command: Commands,
    pub path: Option<PathBuf>,
}

#[derive(Clone, Copy)]
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
                                .join(name.trim_start_matches("/")),
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

                println!("\n\t\x1b[32mCreating \x1b[0mCedar project here");
                println!("\t  -> Generating directories and manifest");

                init(&cwd)?;

                println!("\t  -> Initializing git \n");

                process::Command::new("git")
                    .args(["init", "-b", "main"])
                    .stdout(process::Stdio::null())
                    .spawn()
                    .expect("Git failed to execute, is it installed?")
                    .wait()?;

                println!("\t\x1b[1;32mFinished\x1b[0m");
                Ok(())
            }
            Commands::New => {
                println!(
                    "\n\t\x1b[1;32mCreating \x1b[0m{:?} ({:?})",
                    self.path.as_ref().unwrap().file_name().unwrap(),
                    self.path.as_ref().unwrap()
                );
                println!("\t  -> Generating directories and manifest.");

                let path = self.path.clone().unwrap();

                let path_str = path.clone().into_os_string();

                if !path.is_dir() {
                    fs::create_dir_all(&path)?;
                }

                init(&path)?;

                println!("\t  -> Initializing git \n");

                process::Command::new("git")
                    .args(["init", path_str.to_str().unwrap(), "-b", "main"])
                    .stdout(process::Stdio::null())
                    .spawn()
                    .expect("Git failed to execute, is it installed?")
                    .wait()?;

                println!("\t\x1b[1;32mFinished\x1b[0m");
                Ok(())
            }
            Commands::Build => {
                let cwd = env::current_dir()?;
                build(cwd)?;
                Ok(())
            }
            Commands::Run => {
                let path = env::current_dir()?;

                let manifest_path = path.join("cedar.toml");
                let build_path = path.join("build/");

                let manifest_file = fs::read_to_string(manifest_path)?;
                let manifest = Manifest::parse(&manifest_file)?;

                let output_path = build_path.join(manifest.meta.name);

                build(&path)?;

                let output_str = output_path.to_str().unwrap();

                process::Command::new(output_str)
                    .spawn()
                    .expect("Error: Could not run executable.")
                    .wait()?;

                Ok(())
            }
            Commands::Help => {
                help();
                Ok(())
            }
        }
    }
}

pub fn help() {
    println!(
        "
  A C project manager.

  \x1b[1;32mUsage:\x1b[0m cedar [COMMAND] [OPTIONS]

  \x1b[1;32mCommands:\x1b[0m
    \x1b[1m new      \x1b[0m Creates a new directory with the name/path given and 
                    initializes it as a project.
    \x1b[1m init     \x1b[0m Creates a new project in the current working directory.
    \x1b[1m build    \x1b[0m Compiles the project.
    \x1b[1m run      \x1b[0m Compiles then runs the project.
"
    );
}
