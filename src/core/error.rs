use std::{error::Error, fmt::Display, io, path::PathBuf};

use crate::core::utils::findbyte;

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
    ParseError(*const u8, *const u8, *const u8),
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
            Self::ParseError(start, ptr, end) => {
                let mut str = unsafe {
                    str::from_utf8_unchecked(std::slice::from_raw_parts(
                        *start,
                        *ptr as usize - *start as usize,
                    ))
                }
                .lines()
                .collect::<Vec<_>>();
                let last = unsafe {
                    str::from_utf8_unchecked(std::slice::from_raw_parts(
                        *ptr,
                        findbyte(*ptr, b'\n', *end).unwrap_or(*end) as usize - *ptr as usize,
                    ))
                };
                str.push("");
                let indent = str.last().unwrap().len();
                writeln!(
                    f,
                    "\n{}{}\n{}^\n{}|\n{}Error here",
                    str.join("\n"),
                    last,
                    " ".repeat(indent),
                    " ".repeat(indent),
                    " ".repeat(indent)
                )
            }
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
    NoManifest(String),
    ManifestError(ManifestError),
    LockError(LockError),
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
            Self::NoManifest(s) => writeln!(f, "BuilderError: No Cedar manifest in the path {s}."),
            Self::ManifestError(e) => write!(f, "BuilderError: {e}"),
            Self::LockError(e) => write!(f, "BuilderError: {e}"),
        }
    }
}

impl Error for BuilderError {}

impl From<io::Error> for BuilderError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<ManifestError> for BuilderError {
    fn from(err: ManifestError) -> Self {
        Self::ManifestError(err)
    }
}

impl From<LockError> for BuilderError {
    fn from(err: LockError) -> Self {
        Self::LockError(err)
    }
}

#[derive(Debug)]
pub enum LockError {
    InvalidLockFile,
    NotACFile(PathBuf),
    IoError(io::Error),
}

impl Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLockFile => writeln!(f, "LockError: Invalid lock file."),
            Self::NotACFile(p) => writeln!(
                f,
                "LockError: Attempted to insert a a file without the '.c' or '.h' extension: {p:?}"
            ),
            Self::IoError(e) => writeln!(f, "{e:?}"),
        }
    }
}

impl Error for LockError {}

impl From<io::Error> for LockError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
