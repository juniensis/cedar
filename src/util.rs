use std::{fs, path::Path};

use crate::{error::ProjectError, manifest::Manifest};

/// Accepts an existing path, ensures that the path is empty, then creates
/// the default manifest, src, include, and build folders. Intialization of
/// the git repository occurs in the init command, not the init util, this
/// is to keep command-line options close to the commands module, while having
/// file operations handled in the util module.
///
/// # Arguments
///
/// * 'path' - The empty path to intitialize as a project.
///
pub fn init<P: AsRef<Path>>(path: P) -> Result<(), ProjectError> {
    let path = path.as_ref();

    // Ensure the path exists.
    if !path.is_dir() {
        return Err(ProjectError::InvalidPath(format!("{:?}", path)));
    }

    // Reads the directory and checks that the returned iterator is empty.
    // If not, return an error.
    if path.read_dir()?.next().is_some() {
        return Err(ProjectError::NonEmptyPath(format!("{:?}", path)));
    }

    // Create an array of the paths to build.
    let (src, include, build) = (
        path.join("src/"),
        path.join("include/"),
        path.join("build/"),
    );

    // Create the src, include, and build directories.
    fs::create_dir(&src)?;
    fs::create_dir(&include)?;
    fs::create_dir(&build)?;

    // Create the default main.c file in src.
    let hello_world =
        "#include <stdio.h>\n\nint main() {\n\tprintf(\"Hello World!\");\n\treturn 0;\n}";

    fs::write(src.join("main.c"), hello_world)?;

    // Generate the default manifest.
    let mut manifest = Manifest::default();

    // Set the name equal to the name of the directory, if that fails set the
    // name to "placeholder".
    manifest.meta.name = match path.file_name() {
        Some(name) => name.to_str().unwrap_or("placeholder").to_owned(),
        None => {
            return Err(ProjectError::InvalidPath(format!("{:?}", path)));
        }
    };

    fs::write(path.join("cedar.toml"), manifest.as_string()?)?;

    Ok(())
}
