use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::core::{
    build::{CFile, CHeader, CSource, lock::LockFile, mangle_path},
    error::BuilderError,
    hash::fx_content_hash_words,
    utils::modified,
};

#[derive(Debug)]
pub struct BuildGraph {
    dir: PathBuf,
    lock: LockFile,
    compile: HashSet<Rc<Path>>,
    objs: HashSet<Rc<Path>>,
}

impl BuildGraph {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, BuilderError> {
        let build_dir = root.as_ref().join("build");
        if !build_dir.exists() {
            fs::create_dir_all(&build_dir)?;
        }

        let mut lock = LockFile::new(&build_dir)?;
        let mut sources = Vec::new();
        let mut headers = Vec::new();
        cfile_recurse(&mut headers, &mut sources, root.as_ref())?;

        let mut compile = HashSet::new();
        let mut objs = HashSet::new();

        for source in sources {
            let obj = Rc::<Path>::from(source.obj.as_ref());
            objs.insert(obj);
            if let Some(cmp) = lock.insert_source(source) {
                compile.insert(cmp);
            }
        }

        for header in headers {
            if let Some(deps) = lock.insert_header(header) {
                for dep in deps {
                    compile.insert(dep);
                }
            }
        }

        lock.write()?;

        Ok(Self {
            dir: root.as_ref().to_path_buf(),
            lock,
            compile,
            objs,
        })
    }
    pub fn clean(&self) -> Result<(), BuilderError> {
        for entry in self.dir.join("build/").read_dir()?.flatten() {
            let (pt, ft) = (entry.path(), entry.file_type()?);
            if pt.extension().is_some_and(|ex| ex == "o")
                && !self.objs.contains(&Rc::<Path>::from(
                    pt.with_extension("").file_name().unwrap().as_ref(),
                ))
            {
                fs::remove_file(&pt)?;
                fs::remove_file(pt.with_extension("d"))?;
            }
        }

        Ok(())
    }
    pub fn to_compile(&self) -> Vec<Rc<Path>> {
        self.compile.iter().cloned().collect()
    }
    pub fn to_link(&self) -> Vec<Rc<Path>> {
        self.objs.iter().cloned().collect()
    }
}

fn cfile_recurse<P: AsRef<Path>>(
    hdr: &mut Vec<CHeader>,
    src: &mut Vec<CSource>,
    path: P,
) -> io::Result<()> {
    let path = path.as_ref();
    for entry in path.read_dir()?.flatten() {
        let ft = entry.file_type()?;
        let pt = entry.path();
        if ft.is_dir() {
            cfile_recurse(hdr, src, pt)?;
        } else if ft.is_file() {
            if pt.extension().is_some_and(|ex| ex == "c") {
                let obj = mangle_path(&pt);
                let hash = fx_content_hash_words(&pt);
                let modified = modified(&pt).unwrap();
                src.push(CSource {
                    path: pt,
                    obj,
                    hash,
                    modified,
                })
            } else if pt.extension().is_some_and(|ex| ex == "h") {
                let hash = fx_content_hash_words(&pt);
                let modified = modified(&pt).unwrap();
                hdr.push(CHeader {
                    hash,
                    modified,
                    path: pt,
                    dependents: HashSet::new(),
                })
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod core_build_graph_t {
    use crate::core::build::graph::BuildGraph;

    #[test]
    fn core_build_graph_init_t() {
        let small_example = "./tests/proj/cred_jwerle_b64";
        let graph = BuildGraph::new(small_example).unwrap();
    }
}
