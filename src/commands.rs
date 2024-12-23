use std::{error::Error, fs, path::Path, process, time::Instant};

use crate::{
    error::{BuildError, ProjectError},
    manifest::Manifest,
    util,
};

/// Accepts a path to a Cedar project and searches through the directory for
/// C files and compiles them together into the build directory.
///
/// # Arguments
///
/// * 'path' - The path to the Cedar project to build.
///
pub fn build<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let now = Instant::now();

    let path = path.as_ref();
    let manifest_path = path.join("cedar.toml");
    let src_path = path.join("src/");
    let include_path = path.join("include/");
    let build_path = path.join("build/");

    for path in [&manifest_path, &src_path, &include_path, &build_path] {
        if !path.exists() {
            return Err(Box::new(BuildError::InvalidDirectory));
        }
    }

    let manifest_str = fs::read_to_string(&manifest_path)?;
    let manifest = Manifest::parse(&manifest_str)?;

    println!(
        "\n\t\x1b[1;32mCompiling \x1b[0m{} v{} ({:?})\n",
        manifest.meta.name, manifest.meta.version, &path
    );

    let mut compiler_args: Vec<String> = Vec::new();

    let mut src_files = recursive_file_search(src_path)?;
    let include_files = recursive_file_search(include_path)?;

    src_files.extend_from_slice(&include_files);

    for file in src_files {
        compiler_args.push(file);
    }

    let output_path = build_path.join(manifest.meta.name);
    let output_str = output_path.to_str().unwrap();

    compiler_args.extend_from_slice(&manifest.build.cflags);

    process::Command::new(match manifest.build.compiler.as_str() {
        "GCC" | "gcc" => "gcc",
        "CLANG" | "clang" | "Clang" => "clang",
        _ => return Err(Box::new(BuildError::InvalidCompiler)),
    })
    .args(compiler_args)
    .args(["-o", output_str])
    .spawn()
    .expect("Error: Failed to start compiler.")
    .wait()?;

    let elapsed = now.elapsed();
    println!("\t\x1b[1;32mFinished\x1b[0m in {:.2?}\n", elapsed);

    Ok(())
}

fn recursive_file_search<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::new();
    for file in fs::read_dir(path)? {
        let file_path = file?.path();

        if file_path.is_dir() {
            result.extend_from_slice(&recursive_file_search(file_path)?);
        } else {
            result.push(
                file_path
                    .to_str()
                    .expect("Error: Failed to convert path to string.")
                    .to_owned(),
            )
        }
    }

    Ok(result)
}

pub fn init<P: AsRef<Path>>(path: P, git: bool) -> Result<(), ProjectError> {
    let now = Instant::now();
    let path = path.as_ref();

    let path_str = path.to_str().unwrap();

    println!("\n\t\x1b[32mCreating \x1b[0mCedar project here");
    println!("\t  -> Generating directories and manifest");

    util::init(path)?;

    if git {
        println!("\t  -> Initializing git \n");

        process::Command::new("git")
            .args(["init", path_str, "-b", "main"])
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Git failed to execute, is it installed?")
            .wait()?;
    }

    let elapsed = now.elapsed();
    println!("\t\x1b[1;32mFinished\x1b[0m in {:.2?}\n", elapsed);

    Ok(())
}

pub fn new<P: AsRef<Path>>(path: P, git: bool) -> Result<(), ProjectError> {
    let now = Instant::now();
    let path = path.as_ref();

    let path_str = path.to_str().unwrap();

    println!(
        "\n\t\x1b[1;32mCreating \x1b[0m{} ({})",
        path.file_name().unwrap().to_str().unwrap(),
        path_str
    );

    println!("\t  -> Generating directories and manifest.");

    if !path.is_dir() {
        fs::create_dir_all(path)?;
    }

    util::init(path)?;

    if git {
        println!("\t  -> Initializing git \n");

        process::Command::new("git")
            .args(["init", path_str, "-b", "main"])
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Git failed to execute, is it installed?")
            .wait()?;
    }

    let elapsed = now.elapsed();
    println!("\t\x1b[1;32mFinished\x1b[0m in {:.2?}\n", elapsed);

    Ok(())
}

pub fn run<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();

    let manifest_path = path.join("cedar.toml");
    let build_path = path.join("build/");

    if !manifest_path.is_file() {
        return Err(Box::new(ProjectError::InvalidPath(format!(
            "Missing cedar.toml file in path: {:?}",
            path
        ))));
    }

    if !build_path.is_dir() {
        return Err(Box::new(ProjectError::InvalidPath(format!(
            "Missing build directory in path: {:?}",
            path
        ))));
    }

    let manifest_file = fs::read_to_string(manifest_path)?;
    let manifest = Manifest::parse(&manifest_file)?;

    let output_path = build_path.join(manifest.meta.name);

    build(path)?;

    let output_str = output_path.to_str().unwrap();

    process::Command::new(output_str)
        .spawn()
        .expect("Error: Could not run executable.")
        .wait()?;

    Ok(())
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

  \x1b[1;32mOptions:\x1b[0m
    \x1b[1m --git     \x1b[0m Initializes the project as a git repository when created.
"
    );
}
