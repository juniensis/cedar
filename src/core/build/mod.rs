use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::UNIX_EPOCH,
};

use crate::core::{
    build::{compiler::Compiler, graph::BuildGraph},
    error::BuilderError,
    hash::{K, fx_content_hash_words, fx_hash},
    manifest::Manifest,
};

pub mod compiler;
pub mod graph;
pub mod lock;

#[derive(Debug)]
pub struct Builder {
    root: PathBuf,
    build_dir: PathBuf,
    bin_dir: PathBuf,
    graph: BuildGraph,
    compiler: Compiler,
    manifest: Manifest,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, BuilderError> {
        let root = root.as_ref();
        let manifest_path = root.join("cedar.toml");
        if !manifest_path.exists() {
            return Err(BuilderError::NoManifest(format!(
                "{}",
                root.to_string_lossy()
            )));
        }

        let manifest = Manifest::parse(&fs::read(manifest_path)?)?;
        let bin_dir = root.join(format!("{}", manifest.name));
        let compiler = manifest.compiler()?;
        let graph = BuildGraph::new(root, &bin_dir)?;

        println!(
            "\n  \x1b[1;32mBuilding\x1b[0m {} {}({:?})",
            manifest.name,
            manifest
                .version
                .clone()
                .map(|x| format!("v{x} "))
                .unwrap_or_default(),
            root,
        );

        Ok(Self {
            root: root.to_path_buf(),
            build_dir: root.join("build"),
            bin_dir: root.join(format!("{}", manifest.name)),
            graph,
            compiler,
            manifest,
        })
    }
    pub fn bin_dir(&self) -> PathBuf {
        self.bin_dir.clone()
    }
    pub fn build(&mut self) -> Result<(), BuilderError> {
        if !self.graph.to_compile().is_empty() {
            println!("    \x1b[1;32mCompiling\x1b[0m",);
        }

        let mut children = Vec::new();
        for cmp in self.graph.to_compile() {
            println!("      -> {}", cmp.to_string_lossy());
            let dst = self.root.join("build").join(mangle(&cmp));
            children.push(self.compiler.compile(cmp, dst.into())?);
        }

        for (cmd, mut chld) in children {
            if !chld.wait()?.success() {
                return Err(BuilderError::CompileError((
                    format!("Compiler failed to run {cmd}"),
                    self.build_dir.clone(),
                )));
            }
        }

        self.graph.clean()?;

        if !self.graph.to_link().is_empty() {
            println!("    \x1b[1;32mLinking\x1b[0m");
            let link = self
                .graph
                .to_link()
                .iter()
                .map(|x| {
                    let dir = self.build_dir.join(x);
                    println!("      -> {}.o", dir.as_os_str().to_string_lossy());
                    dir
                })
                .collect::<Vec<_>>();

            self.compiler.link(&link, &self.bin_dir)?;
        }

        Ok(())
    }
}

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
        "{:016x}",
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
    pub fn from_path<P: AsRef<Path>>(path: P, build: P) -> Result<Self, BuilderError> {
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
    pub fn add_dependents<P: AsRef<Path>>(&mut self, build: P) -> Result<(), BuilderError> {
        dep_recurse(self, build)?;
        Ok(())
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

fn dep_recurse<P: AsRef<Path>>(out: &mut CHeader, path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    for entry in path.read_dir()?.flatten() {
        let pt = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            dep_recurse(out, path)?;
        } else if ft.is_file() && pt.extension().is_some_and(|ex| ex == "d") {
            let dfile = fs::read_to_string(&pt)?;
            if let Some(split) = dfile.split(':').next_back() {
                let mut iter = split.split_ascii_whitespace().filter(|x| x.trim() != "\\");
                if let Some(src) = iter.next()
                    && src.ends_with(".c")
                {
                    for header in iter.map(|x| PathBuf::from(format!("./{}", x))) {
                        if header == out.path {
                            out.dependents.insert(Rc::<Path>::from(PathBuf::from(src)));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod core_build_t {
    use crate::core::build::Builder;

    #[test]
    fn builder_t() {
        let mut builder = Builder::new("./tests/proj/cred_attractivechaos_klib").unwrap();
        builder.build().unwrap();
    }
}
