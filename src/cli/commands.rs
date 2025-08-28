use std::{fs, path::Path, process, time::Instant};

use crate::{
    cli::error::CliError,
    core::{build::Builder, error::BuilderError, manifest::Manifest},
};

pub fn help() {
    println!(
        "
  A C project manager.

  \x1b[1;32mUsage:\x1b[0m cedar [COMMAND] [OPTIONS]

  \x1b[1;32mCommands:\x1b[0m
    \x1b[1m new      \x1b[0m Creates a new project under the given name.
    \x1b[1m init     \x1b[0m Creates a new project in the current working directory.
    \x1b[1m build    \x1b[0m Compiles the project.
    \x1b[1m run      \x1b[0m Compiles then runs the project.

  \x1b[1;32mOptions:\x1b[0m
    \x1b[1m --git     \x1b[0m Initializes the project as a git repository when created.
"
    );
}

pub fn init<P: AsRef<Path>>(path: P, git: bool) -> Result<(), CliError> {
    let now = Instant::now();
    println!(
        "\n\t\x1b[32mCreating \x1b[0mCedar project at {}",
        path.as_ref().to_string_lossy()
    );
    println!("\t  -> Generating directories and manifest");
    let path = path.as_ref();
    let name = path
        .file_name()
        .map(|x| x.to_string_lossy())
        .unwrap_or("project".into());

    let manifest = Manifest::new(&name)?;
    if !path.exists() {
        return Err(CliError::InitInNonExistentPath(format!("{path:?}")));
    }

    manifest.write(path.join("cedar.toml"))?;

    fs::create_dir(path.join("src"))?;

    let initial = b"int main() { return 0; }";
    fs::write(path.join("src/main.c"), initial)?;

    if git {
        println!("\t  -> Initializing git \n");

        process::Command::new("git")
            .args(["init", &path.to_string_lossy(), "-b", "main"])
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Git failed to execute.")
            .wait()?;
    }

    let elapsed = now.elapsed();
    println!("\t\x1b[1;32mFinished\x1b[0m in {:.2?}\n", elapsed);

    Ok(())
}

pub fn build<P: AsRef<Path>>(path: P) -> Result<(), BuilderError> {
    let now = Instant::now();

    let mut builder = Builder::new(path.as_ref())?;
    builder.build()?;

    let elapsed = now.elapsed();
    println!("\n  \x1b[1;32mFinished\x1b[0m in {:.2?}\n", elapsed);

    Ok(())
}

pub fn new<P: AsRef<Path>>(path: P, git: bool) -> Result<(), CliError> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    init(path, git)
}

pub fn run<P: AsRef<Path>>(path: P) -> Result<(), BuilderError> {
    let now = Instant::now();
    let mut builder = Builder::new(path.as_ref())?;
    let bin_dir = builder.bin_dir();
    builder.build()?;
    println!(
        "  \x1b[1;32mRunning\x1b[0m {}",
        bin_dir.file_name().unwrap_or_default().to_string_lossy()
    );
    println!();
    process::Command::new(bin_dir.to_string_lossy().as_ref())
        .spawn()
        .expect("Error: Failed to run executable.")
        .wait()?;

    let elapsed = now.elapsed();
    println!("\n\n  \x1b[1;32mFinished\x1b[0m in {:.2?}\n", elapsed);

    Ok(())
}

#[cfg(test)]
mod cli_commands_t {
    use crate::cli::commands::{build, init, new, run};

    #[ignore]
    #[test]
    fn init_t() {
        init("./tests/proj/init_t", false).unwrap();
    }

    #[ignore]
    #[test]
    fn new_t() {
        new("./tests/proj/new_t", false).unwrap();
    }

    #[ignore]
    #[test]
    fn build_t() {
        build("./tests/proj/init_t").unwrap();
    }

    #[test]
    fn run_t() {
        run("./tests/proj/init_t/").unwrap();
    }
}
