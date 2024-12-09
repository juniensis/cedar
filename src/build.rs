use std::time::Instant;
use std::{error::Error, fmt::Display, fs, path::Path, process};

use crate::structure::manifest::Manifest;

#[derive(Debug)]
pub enum BuildError {
    InvalidDirectory,
    InvalidCompiler,
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::InvalidDirectory => writeln!(f, "Error: Project has invalid structure."),
            BuildError::InvalidCompiler => {
                writeln!(f, "Error: Compiler given in the manifest is invalid.")
            }
        }
    }
}

impl Error for BuildError {}

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

    let src_files = fs::read_dir(src_path)?;
    let include_files = fs::read_dir(include_path)?;

    for files in src_files.chain(include_files) {
        compiler_args.push(
            files?
                .path()
                .to_str()
                .expect("Error: Failed to convert path to string while building.")
                .to_owned(),
        );
    }

    let output_path = build_path.join(manifest.meta.name);
    let output_str = output_path.to_str().unwrap();

    compiler_args.extend_from_slice(&manifest.build.cflags);

    process::Command::new(match manifest.build.compiler.as_str() {
        "GCC" | "gcc" => "gcc",
        "CLANG" | "clang" | "Clang" => todo!(),
        _ => return Err(Box::new(BuildError::InvalidDirectory)),
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
