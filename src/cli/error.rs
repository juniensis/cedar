use std::{error::Error, fmt::Display, io};

use crate::core::error::{BuilderError, ManifestError};

#[derive(Debug)]
pub enum CliError {
    IoError(io::Error),
    MissingArgument(&'static str),
    InvalidCommand,
    InitInNonEmptyPath(String),
    InitInNonExistentPath(String),
    ManifestError(ManifestError),
    BuilderError(BuilderError),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => writeln!(f, "CliError: {e}"),
            Self::MissingArgument(fl) => writeln!(f, "CliError: Missing argument, {fl}"),
            Self::InvalidCommand => writeln!(f, "CliError: Invalid command."),

            Self::InitInNonEmptyPath(p) => writeln!(
                f,
                "CliError: Attempted to initialize in a non-empty path, {p}"
            ),
            Self::InitInNonExistentPath(p) => writeln!(
                f,
                "CliError: attempted to initialize in a non-existent path, {p}"
            ),
            Self::ManifestError(m) => {
                writeln!(f, "CliError: Failed to generate or parse manifest, {m}")
            }
            Self::BuilderError(e) => writeln!(f, "ClieError: Failed during build, {e}"),
        }
    }
}

impl Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<ManifestError> for CliError {
    fn from(err: ManifestError) -> Self {
        Self::ManifestError(err)
    }
}

impl From<BuilderError> for CliError {
    fn from(err: BuilderError) -> Self {
        Self::BuilderError(err)
    }
}
