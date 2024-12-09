use std::{fs, path::Path};

use crate::structure::project::ProjectError;

use super::manifest::Manifest;

/// Ensures the current path is empty, then creates the default manifest,
/// the src, include, and build folders, and initializes a git repository.
///
/// # Arguments
///
/// * 'path' - The empty path to initialize as a project, any type that can be
///         coerced into a path.
///         
pub fn init<P: AsRef<Path>>(path: P) -> Result<(), ProjectError> {
    let path = path.as_ref();

    // Ensure the path is an existing directory.
    if !path.is_dir() {
        return Err(ProjectError::InvalidPath(format!("{:?}", path)));
    }

    // Ensure the path is empty.
    if path.read_dir()?.next().is_some() {
        return Err(ProjectError::NonEmptyPath(format!("{:?}", path)));
    }

    let (src, include, build) = (
        path.join("src/"),
        path.join("include/"),
        path.join("build/"),
    );

    // Create src, include, build, and .cedar directories.
    fs::create_dir(&src)?;
    fs::create_dir(&include)?;
    fs::create_dir(&build)?;

    // Create default main.c file in src.
    let hello_world =
        "#include <stdio.h>\n\nint main() {\n\tprintf(\"Hello World!\");\n\treturn 0;\n}";

    fs::write(src.join("main.c"), hello_world)?;

    let mut manifest = Manifest::default();

    manifest.meta.name = match path.file_name() {
        Some(name) => name.to_str().unwrap_or("placeholder").to_owned(),
        None => {
            return Err(ProjectError::InvalidPath(format!("{:?}", path)));
        }
    };

    fs::write(path.join("cedar.toml"), manifest.as_string()?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::init;

    #[test]
    fn test_init() {
        init("./tests/project/").unwrap();
    }
}
