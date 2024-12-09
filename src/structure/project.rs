use std::{error::Error, fmt::Display, io};

/// A custom error type for errors when building a project.
///
/// # Members
///
/// * 'NoManifest' - Used when there is no manifest in the directory given.
/// * 'InvalidPath' - Used when the given path does not exist. Holds a String
///         primarily to give the path that caused the error, however, a String
///         is used to give flexibility to the information passed.
/// * 'IoError' - A wrapper for std::io::Error to allow for error propogation
///         within functions that return ProjectError without using a Box.
/// * 'NonEmptyPath' - Used when the path given is not empty, this will likely
///         be handled by asking the user to confirm overwriting the directory.
///
#[derive(Debug)]
pub enum ProjectError {
    InvalidManifest,
    InvalidPath(String),
    IoError(io::Error),
    NonEmptyPath(String),
}

impl Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidManifest => writeln!(
                f,
                "Error: Manifest is either invalid, failed to be created, or does not exist."
            ),
            Self::InvalidPath(s) => writeln!(f, "Error: Invalid path given. \n {:?}", s),
            Self::IoError(e) => writeln!(f, "Error: Project caused an std::io::Error. \n {}", e),
            Self::NonEmptyPath(s) => writeln!(f, "Error: Path given is not empty. \n {}", s),
        }
    }
}

impl Error for ProjectError {}

impl From<io::Error> for ProjectError {
    fn from(err: io::Error) -> Self {
        ProjectError::IoError(err)
    }
}
