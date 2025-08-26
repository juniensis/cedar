use std::{
    collections::HashSet,
    fmt::Display,
    path::{Path, PathBuf},
    rc::Rc,
    time::UNIX_EPOCH,
};

use crate::core::{
    error::BuilderError,
    hash::{K, fx_content_hash_words, fx_hash},
};

pub mod compiler;
pub mod graph;
pub mod lock;

#[derive(Debug, Clone)]
pub struct CSource {
    pub path: PathBuf,
    // What the .o file would be, does not imply existence.
    pub obj: PathBuf,
    pub hash: u64,
    pub modified: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CHeader {
    pub path: PathBuf,
    pub hash: u64,
    pub modified: u64,
    pub dependents: HashSet<Rc<Path>>,
}

impl PartialEq for CSource {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.modified == other.modified && self.path == other.path
    }
}

impl Eq for CSource {}

#[inline]
pub fn mangle<P: AsRef<Path>>(path: P) -> String {
    format!(
        "{:016x}.o",
        fx_hash(path.as_ref().as_os_str().as_encoded_bytes())
    )
}

#[inline]
pub fn mangle_path<P: AsRef<Path>>(path: P) -> PathBuf {
    PathBuf::from(mangle(path))
}

pub enum CFile<'a> {
    Source(&'a CSource),
    Header(&'a CHeader),
}

impl CSource {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, BuilderError> {
        let path = path.as_ref();
        let obj = mangle_path(path);
        let hash = fx_content_hash_words(path);
        let modified = path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            path: path.to_path_buf(),
            obj,
            hash,
            modified,
        })
    }
    pub fn update(&mut self) -> Result<(), BuilderError> {
        self.hash = fx_content_hash_words(&(self.path));
        self.modified = self
            .path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }
    pub fn as_cfile(&'_ self) -> CFile<'_> {
        CFile::Source(self)
    }
}

impl CHeader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, BuilderError> {
        let path = path.as_ref();
        let hash = fx_content_hash_words(path);
        let modified = path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            path: path.to_path_buf(),
            hash,
            modified,
            dependents: HashSet::new(),
        })
    }
    pub fn update(&mut self) -> Result<(), BuilderError> {
        self.hash = fx_content_hash_words(&(self.path));
        self.modified = self
            .path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }
    pub fn as_cfile(&'_ self) -> CFile<'_> {
        CFile::Header(self)
    }
    pub fn add_dependent<P: AsRef<Path>>(&mut self, path: P) {
        let rc = Rc::<Path>::from(path.as_ref());
        self.dependents.insert(rc);
    }
}

impl Display for CSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CSource:\n  path: {}\n  obj:  {}\n  hash:  {}\n  modified:  {}",
            self.path.as_os_str().to_string_lossy(),
            self.obj.as_os_str().to_string_lossy(),
            self.hash,
            self.modified,
        )
    }
}

impl Display for CHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CHeader:\n  path: {}\n  hash: {}\n  modified: {}\n  dependents:",
            self.path.as_os_str().to_string_lossy(),
            self.hash,
            self.modified
        )?;

        for source in &self.dependents {
            writeln!(f, " -> {}", source.to_string_lossy())?;
        }

        Ok(())
    }
}
