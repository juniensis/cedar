use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
    time::UNIX_EPOCH,
};

use crate::core::{
    build::{CFile, CHeader, CSource, mangle_path},
    error::LockError,
    hash::fx_content_hash_words,
    manifest::toml::{Value, toml_parse},
};

#[derive(PartialEq, Eq)]
pub struct LockFile {
    path: PathBuf,
    sources: HashMap<Rc<Path>, CSource>,
    headers: HashMap<Rc<Path>, CHeader>,
}

impl LockFile {
    #[inline]
    pub fn new<P: AsRef<Path>>(build_dir: P) -> Result<Self, LockError> {
        let path = build_dir.as_ref().join("cedar.lock");
        if path.exists() {
            let bytes = fs::read(&path)?;
            Self::deserialize(path, &bytes)
        } else {
            fs::write(&path, [0x0a])?;

            let sources = HashMap::new();
            let headers = HashMap::new();
            Ok(Self {
                path,
                sources,
                headers,
            })
        }
    }
    /// Insert the given source file and return if the file still needs to
    /// be compiled.
    #[inline]
    pub fn insert_source(&mut self, src: CSource) -> Option<Rc<Path>> {
        let rc = Rc::<Path>::from(src.path.as_ref());
        let file = src.as_cfile();
        if self.contains(&file) {
            if !self.hash_matches(&file) {
                self.sources.insert(rc.clone(), src);
                Some(rc)
            } else {
                self.sources.insert(rc, src);
                None
            }
        } else {
            self.sources.insert(rc.clone(), src);
            Some(rc)
        }
    }
    /// Insert a header and return if the headers dependents need to be
    /// compiled.
    #[inline]
    pub fn insert_header(&mut self, hdr: CHeader) -> Option<Vec<Rc<Path>>> {
        let rc = Rc::<Path>::from(hdr.path.as_ref());
        let file = hdr.as_cfile();
        if self.contains(&file) {
            if !self.hash_matches(&file) {
                let ret = hdr.dependents.clone();
                self.headers.insert(rc.clone(), hdr);
                Some(ret.into_iter().collect())
            } else {
                self.headers.insert(rc, hdr);
                None
            }
        } else {
            let ret = hdr.dependents.clone();
            self.headers.insert(rc, hdr);
            Some(ret.into_iter().collect())
        }
    }
}

impl LockFile {
    #[inline]
    pub fn path_contains<P: AsRef<Path>>(&self, path: P) -> bool {
        self.sources.contains_key(&Rc::from(path.as_ref()))
            || self.headers.contains_key(&Rc::from(path.as_ref()))
    }
    #[inline]
    pub fn path_hash_matches<P: AsRef<Path>>(&self, path: P) -> bool {
        let rc = Rc::from(path.as_ref());
        let hash = fx_content_hash_words(&rc);
        self.sources.get(&rc).is_some_and(|src| src.hash == hash)
            || self.headers.get(&rc).is_some_and(|hdr| hdr.hash == hash)
    }
    #[inline]
    pub fn path_is_newer<P: AsRef<Path>>(&self, path: P) -> bool {
        let rc = Rc::<Path>::from(path.as_ref());
        path.as_ref().metadata().is_ok_and(|mt| {
            mt.modified().is_ok_and(|md| {
                md.duration_since(UNIX_EPOCH).is_ok_and(|ds| {
                    ds.as_secs()
                        > if let Some(src) = self.sources.get(&rc) {
                            src.modified
                        } else if let Some(hdr) = self.headers.get(&rc) {
                            hdr.modified
                        } else {
                            return false;
                        }
                })
            })
        })
    }
    #[inline]
    pub fn contains(&self, file: &CFile) -> bool {
        if let CFile::Source(src) = file {
            self.path_contains(&src.path)
        } else if let CFile::Header(hdr) = file {
            self.path_contains(&hdr.path)
        } else {
            false
        }
    }
    #[inline]
    pub fn hash_matches(&self, file: &CFile) -> bool {
        if let CFile::Source(src) = file {
            self.sources
                .get(&Rc::<Path>::from(src.path.as_ref()))
                .is_some_and(|s| s.hash == src.hash)
        } else if let CFile::Header(hdr) = file {
            self.headers
                .get(&Rc::<Path>::from(hdr.path.as_ref()))
                .is_some_and(|h| h.hash == hdr.hash)
        } else {
            false
        }
    }
    #[inline]
    pub fn is_newer(&self, file: &CFile) -> bool {
        if let CFile::Source(src) = file {
            self.sources
                .get(&Rc::<Path>::from(src.path.as_ref()))
                .is_some_and(|s| s.modified < src.modified)
        } else if let CFile::Header(hdr) = file {
            self.headers
                .get(&Rc::<Path>::from(hdr.path.as_ref()))
                .is_some_and(|h| h.modified < hdr.modified)
        } else {
            false
        }
    }
}

impl LockFile {
    pub fn serialize(&self) -> Result<Vec<u8>, LockError> {
        let mut out = String::new();
        for (path, header) in &self.headers {
            let path_str = path.to_string_lossy();
            let dep_str = format!(
                "dependents = [{}]",
                header
                    .dependents
                    .iter()
                    .map(|dep| {
                        let str = dep.to_string_lossy();
                        format!("\"{str}\"")
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            out.push_str(
                format!(
                    "[{path_str}]\nhash = {}\nmodified = {}\n{dep_str}\n\n",
                    header.hash, header.modified
                )
                .as_str(),
            );
        }
        for (src_path, source) in &self.sources {
            let path_str = src_path.to_string_lossy();
            out.push_str(
                format!(
                    "[{path_str}]\nobj = \"{}\"\nhash = {}\nmodified = {}\n\n",
                    source.obj.to_string_lossy(),
                    source.hash,
                    source.modified
                )
                .as_str(),
            );
        }

        Ok(out.as_bytes().to_vec())
    }
    pub fn deserialize(path: PathBuf, bytes: &[u8]) -> Result<Self, LockError> {
        let tables = toml_parse(bytes);

        let mut headers = HashMap::new();
        let mut sources = HashMap::new();

        for table in tables {
            let file_path = PathBuf::from(table.name.as_ref());
            let path_rc = Rc::<Path>::from(file_path.as_ref());

            if path_rc.extension().is_some_and(|ex| ex == "c") {
                let obj_val = table
                    .get("obj")
                    .expect("Someone messed with the lock file :(");
                let hash_val = table
                    .get("hash")
                    .expect("Someone messed with the lock file :(");
                let modified_val = table
                    .get("modified")
                    .expect("Someone messed with the lock file :(");

                if let (Value::String(obj), Value::String(hash), Value::String(modified)) =
                    (obj_val, hash_val, modified_val)
                {
                    sources.insert(
                        path_rc,
                        CSource {
                            path: file_path,
                            obj: PathBuf::from(obj),
                            hash: hash
                                .parse::<u64>()
                                .expect("Someone messed with the lock file :("),
                            modified: modified
                                .parse::<u64>()
                                .expect("Someone messed with the lock file :("),
                        },
                    );
                }
            } else if path_rc.extension().is_some_and(|ex| ex == "h") {
                let hash_val = table
                    .get("hash")
                    .expect("Someone messed with the lock file :(");
                let modified_val = table
                    .get("modified")
                    .expect("Someone messed with the lock file :(");
                let deps_val = table
                    .get("dependents")
                    .expect("Someone messed with the lock file :(");

                if let (Value::String(hash), Value::String(modified), Value::List(deps)) =
                    (hash_val, modified_val, deps_val)
                {
                    headers.insert(
                        path_rc,
                        CHeader {
                            path: file_path,
                            hash: hash
                                .parse::<u64>()
                                .expect("Someone messed with the lock file :("),
                            modified: modified
                                .parse::<u64>()
                                .expect("Someone messed with the lock file :("),
                            dependents: deps
                                .iter()
                                .map(|val| {
                                    if let Value::String(p) = val {
                                        Rc::<Path>::from(PathBuf::from(p).as_ref())
                                    } else {
                                        unreachable!()
                                    }
                                })
                                .collect::<HashSet<Rc<Path>>>(),
                        },
                    );
                }
            }
        }

        Ok(Self {
            path,
            headers,
            sources,
        })
    }
    pub fn write(&self) -> Result<(), LockError> {
        let bytes = self.serialize()?;
        fs::write(&self.path, &bytes)?;
        Ok(())
    }
}

impl Debug for LockFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "LockFile: {:?}\nHeaders:", self.path)?;
        for header in self.headers.values() {
            writeln!(f, "{}", header)?;
        }
        for source in self.sources.values() {
            writeln!(f, "{}", source)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod core_build_lock_t {
    use std::{
        collections::HashSet,
        fs,
        path::{Path, PathBuf},
        rc::Rc,
        thread,
        time::Duration,
    };

    use crate::core::{
        build::{CHeader, CSource, lock::LockFile, mangle_path},
        manifest::toml::toml_parse,
    };

    #[ignore = "Sleeps"]
    #[test]
    fn lock_bools_t() {
        let root = PathBuf::from("./tests/lock_t/");
        let (src_path, hdr_path) = (root.join("src_t.c"), root.join("hdr_t.h"));
        fs::remove_file(&src_path).unwrap();
        fs::remove_file(&hdr_path).unwrap();
        fs::remove_file("./tests/lock_t/cedar.lock").unwrap();

        fs::write(
            &src_path,
            b"#include <stdio.h>\n\nint main() {{\nprintf(\"original\");\n  return 0;",
        )
        .unwrap();
        fs::write(&hdr_path, b"int add(int a, int b);").unwrap();

        let mut src = CSource::from_path(&src_path).unwrap();
        let mut hdr = CHeader::from_path(&hdr_path).unwrap();

        let mut lock = LockFile::new(root).unwrap();
        assert!(!lock.contains(&src.as_cfile()));
        assert!(!lock.contains(&hdr.as_cfile()));
        assert!(!lock.path_contains(&src_path));
        assert!(!lock.path_contains(&hdr_path));

        lock.insert_source(src.clone());
        lock.insert_header(hdr.clone());

        assert!(lock.hash_matches(&src.as_cfile()));
        assert!(lock.hash_matches(&hdr.as_cfile()));
        assert!(!lock.is_newer(&src.as_cfile()));
        assert!(!lock.is_newer(&hdr.as_cfile()));
        assert!(lock.path_hash_matches(&src_path));
        assert!(lock.path_hash_matches(&hdr_path));
        assert!(!lock.path_is_newer(&src_path));
        assert!(!lock.path_is_newer(&hdr_path));
        thread::sleep(Duration::from_secs(1));

        fs::write(
            &src_path,
            b"#include <stdio.h>\n\nint main() {{\nprintf(\"changed\");\n  return 0;",
        )
        .unwrap();
        fs::write(&hdr_path, b"int sub(int a, int b);").unwrap();

        src.update().unwrap();
        hdr.update().unwrap();

        assert!(lock.contains(&src.as_cfile()));
        assert!(lock.contains(&hdr.as_cfile()));
        assert!(!lock.hash_matches(&src.as_cfile()));
        assert!(!lock.hash_matches(&hdr.as_cfile()));
        assert!(lock.is_newer(&src.as_cfile()));
        assert!(lock.is_newer(&hdr.as_cfile()));
        assert!(lock.path_contains(&src_path));
        assert!(lock.path_contains(&hdr_path));
        assert!(!lock.path_hash_matches(&src_path));
        assert!(!lock.path_hash_matches(&hdr_path));
        assert!(lock.path_is_newer(&src_path));
        assert!(lock.path_is_newer(&hdr_path));
    }
    #[test]
    fn lock_serialize_t() {
        let mut lock = LockFile::new("./tests/lock_t/").unwrap();
        let hdr_1 = CHeader {
            path: PathBuf::from("./tests/lock_t/src/hdr_1.h"),
            hash: 3987987987987987,
            modified: 987987987987,
            dependents: HashSet::from([
                Rc::<Path>::from(PathBuf::from("./tests/lock_t/src/dep_1.c").as_ref()),
                Rc::<Path>::from(PathBuf::from("./tests/lock_t/src/dep_2.c").as_ref()),
                Rc::<Path>::from(PathBuf::from("./tests/lock_t/src/dep_3.c").as_ref()),
            ]),
        };
        let src_1 = CSource {
            path: PathBuf::from("./tests/lock_t/src/dep_1.c"),
            obj: mangle_path("./tests/lock_t/src/dep_1.c"),
            hash: 890123098123089,
            modified: 789234897234,
        };
        let src_2 = CSource {
            path: PathBuf::from("./tests/lock_t/src/dep_2.c"),
            obj: mangle_path("./tests/lock_t/src/dep_2.c"),
            hash: 890123098123089,
            modified: 789234897234,
        };
        let src_3 = CSource {
            path: PathBuf::from("./tests/lock_t/src/dep_3.c"),
            obj: mangle_path("./tests/lock_t/src/dep_3.c"),
            hash: 890123098123089,
            modified: 789234897234,
        };

        lock.insert_header(hdr_1);
        lock.insert_source(src_1);
        lock.insert_source(src_2);
        lock.insert_source(src_3);

        let t = lock.serialize().unwrap();
        let toml = toml_parse(&t);

        let deser = LockFile::deserialize(PathBuf::from("./tests/lock_t/cedar.lock"), &t).unwrap();
        assert_eq!(deser, lock);
    }
}
