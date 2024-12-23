use std::{error, fmt::Display, io};

use toml::{de, ser};

#[derive(Debug)]
pub enum ManifestError {
    InvalidManifest(de::Error),
    SerializeError(ser::Error),
}

#[derive(Debug)]
pub enum BuildError {
    InvalidDirectory,
    InvalidCompiler,
}

#[derive(Debug)]
pub enum CliError {
    InvalidCommand,
    MissingArgument(&'static str),
}

#[derive(Debug)]
pub enum ProjectError {
    ManifestError(ManifestError),
    InvalidPath(String),
    NonEmptyPath(String),
    IoError(io::Error),
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidManifest(e) => {
                write!(f, "Error: Failed to read manifest.\n {e}")
            }
            Self::SerializeError(e) => {
                write!(f, "Error: Failed to create manifest.\n {e}")
            }
        }
    }
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

impl Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManifestError(e) => writeln!(f, "{e}"),
            Self::InvalidPath(s) => writeln!(f, "Error: Invalid path given. \n {:?}", s),
            Self::IoError(e) => writeln!(f, "Error: Project caused an std::io::Error. \n {}", e),
            Self::NonEmptyPath(s) => writeln!(f, "Error: Path given is not empty. \n {}", s),
        }
    }
}

impl error::Error for BuildError {}
impl error::Error for ManifestError {}
impl error::Error for CliError {}
impl error::Error for ProjectError {}

impl From<ManifestError> for ProjectError {
    fn from(err: ManifestError) -> Self {
        ProjectError::ManifestError(err)
    }
}

impl From<io::Error> for ProjectError {
    fn from(err: io::Error) -> Self {
        ProjectError::IoError(err)
    }
}

impl From<de::Error> for ManifestError {
    fn from(value: de::Error) -> Self {
        Self::InvalidManifest(value)
    }
}

impl From<ser::Error> for ManifestError {
    fn from(value: ser::Error) -> Self {
        Self::SerializeError(value)
    }
}
