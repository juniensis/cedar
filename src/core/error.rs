use std::{error::Error, fmt::Display, io};

#[derive(Debug)]
pub enum WalkerError {
    InvalidRoot,
}

impl Display for WalkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoot => writeln!(f, "WalkerError: Invalid root directory."),
        }
    }
}

impl Error for WalkerError {}

#[derive(Debug)]
pub enum ManifestError {
    MissingName,
    MissingCompiler,
    Invalid,
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingName => writeln!(
                f,
                "Failed to parse manifest, a name field is required: \"name = \"main\"\"."
            ),
            Self::MissingCompiler => {
                writeln!(f, "Failed to parse manifest, no C compiler is specified.")
            }
            Self::Invalid => writeln!(f, "Failed to parse manifest for unknown reasons."),
        }
    }
}

impl Error for ManifestError {}

#[derive(Debug)]
pub enum BuilderError {
    FailedToDetectCompiler,
    InvalidCompiler(String),
    CompileError(String),
    IoError(io::Error),
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToDetectCompiler => writeln!(
                f,
                "BuilderError: Failed to detect valid compiler, currently only clang and gcc are supported."
            ),
            Self::InvalidCompiler(c) => writeln!(
                f,
                "BuilderError: Given invalid compiler '{c}', currently only clang and gcc are supported."
            ),
            Self::CompileError(e) => writeln!(f, "BuilderError: Failed to compile, {e}"),
            Self::IoError(e) => writeln!(f, "{e}"),
        }
    }
}

impl Error for BuilderError {}

impl From<io::Error> for BuilderError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
