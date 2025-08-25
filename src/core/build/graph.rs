use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
    time::UNIX_EPOCH,
};

use crate::core::hash::fx_content_hash_words;

pub enum MetaData {
    Source {
        modified: u64,
        hash: u64,
    },
    Header {
        modified: u64,
        hash: u64,
        dependents: Vec<Rc<Path>>,
    },
}

pub struct BuildGraph {
    src_dir: PathBuf,
    build_dir: PathBuf,
    lock_dir: PathBuf,
    nodes: HashMap<PathBuf, MetaData>,
}

impl BuildGraph {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let src_dir = root.as_ref().join("src");
        let build_dir = root.as_ref().join("build");
        let lock_dir = build_dir.join("cedar.lock");
        let mut nodes = HashMap::new();

        fn rec<P: AsRef<Path>>(out: &mut HashMap<PathBuf, MetaData>, cur: P) -> io::Result<()> {
            let path = cur.as_ref();
            for entry in path.read_dir()?.flatten() {
                let tp = entry.file_type()?;
                let pt = entry.path();
                if tp.is_dir() {
                    rec(out, pt)?;
                } else if tp.is_file() {
                    if pt.extension().is_some_and(|ex| ex == ".c") {
                        let hash = fx_content_hash_words(&pt);
                        let modified = entry
                            .metadata()?
                            .modified()?
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        out.insert(pt, MetaData::Source { modified, hash });
                    } else if pt.extension().is_some_and(|ex| ex == ".h") {
                        let hash = fx_content_hash_words(&pt);
                        let modified = entry
                            .metadata()?
                            .modified()?
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        out.insert(
                            pt,
                            MetaData::Header {
                                modified,
                                hash,
                                dependents: Vec::new(),
                            },
                        );
                    }
                }
            }
            Ok(())
        }

        rec(&mut nodes, root).unwrap();

        Self {
            src_dir,
            build_dir,
            lock_dir,
            nodes,
        }
    }
}
