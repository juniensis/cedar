use std::{
    fs, hash, io,
    path::{Path, PathBuf},
    rc::Rc,
    time::UNIX_EPOCH,
};

use ahash::AHashMap;

use crate::core::utils::{findbyte, fxhash, resolve_path, walk_dir};

// Recompile every changed file, and every file that depends on it if the
// header has changed.
#[derive(Debug)]
pub struct CSource {
    path: Rc<Path>,
    modified: u64,
    content: u64,
}

#[derive(Debug)]
pub struct CHeader {
    path: Rc<Path>,
    modified: u64,
    content: u64,
    dependents: Vec<Rc<Path>>,
}

#[derive(Debug)]
pub struct DependencyGraph {
    sources: AHashMap<Rc<Path>, CSource>,
    headers: AHashMap<Rc<Path>, CHeader>,
    dir: PathBuf,
}

impl DependencyGraph {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            sources: AHashMap::new(),
            headers: AHashMap::new(),
            dir: path.as_ref().to_path_buf(),
        }
    }
    pub fn build<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let build_dir = path.join("build");
        let files = walk_dir(path)?;
        let files_mod = files.iter().map(|pt| {
            let meta = &pt.metadata().unwrap();
            let modified = meta
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            (pt, modified)
        });

        let mut sourcemap = AHashMap::new();
        let mut headermap = AHashMap::new();

        for (file, modified) in files_mod {
            if file.extension().is_some_and(|ex| ex == "c") {
                let bytes = fs::read(file)?;
                let rcpath = Rc::<Path>::from(file.as_ref());
                let content = fxhash(&bytes);
                sourcemap.insert(
                    rcpath.clone(),
                    CSource {
                        path: rcpath,
                        modified,
                        content,
                    },
                );
            } else if file.extension().is_some_and(|ex| ex == "h") {
                let bytes = fs::read(file)?;
                let rcpath = Rc::<Path>::from(file.as_ref());
                let content = fxhash(&bytes);
                headermap.insert(
                    rcpath.clone(),
                    CHeader {
                        path: rcpath,
                        modified,
                        content,
                        dependents: Vec::new(),
                    },
                );
            }
        }

        let dfiles = walk_dir(&build_dir)?;
        let mut alldeps = Vec::with_capacity(dfiles.len());
        for pt in dfiles {
            if pt.extension().is_some_and(|ex| ex == "d") {
                let contents = fs::read(&pt)?;
                let mut ptr = contents.as_ptr();
                let end = unsafe { ptr.add(contents.len()) };
                let mut deps: Vec<Rc<Path>> = Vec::new();
                while let Some((fs, _)) = findbyte(ptr, b' ', end) {
                    ptr = fs;
                    unsafe {
                        let mut rdr;
                        let start = loop {
                            rdr = *ptr;
                            if rdr < 33 || rdr == b'\\' {
                                ptr = ptr.add(1);
                                continue;
                            } else {
                                break ptr;
                            }
                        };
                        if let Some((end, len)) = findbyte(start, b' ', end) {
                            let slice = std::slice::from_raw_parts(start, len);
                            let str = resolve_path(str::from_utf8_unchecked(slice).trim());
                            deps.push(Rc::from(str.as_ref()));
                            ptr = end.add(1);
                        } else {
                            break;
                        }
                    }
                }
                let last = resolve_path(
                    unsafe {
                        str::from_utf8_unchecked(std::slice::from_raw_parts(
                            ptr,
                            end as usize - ptr as usize,
                        ))
                    }
                    .trim(),
                );
                deps.push(Rc::from(last.as_ref()));
                alldeps.push(deps)
            }
        }

        for deps in &alldeps {
            for header in deps[1..].iter() {
                if let Some(head) = headermap.get_mut(header) {
                    head.dependents.push(deps[0].clone());
                }
            }
        }

        let mut ser = String::new();

        for (key, entry) in &headermap {
            let pt = key.to_string_lossy();
            ser.push_str(&format!(
                "<{pt}\x1f{}\x1f{}\n",
                entry.content, entry.modified
            ));
            for dep in &entry.dependents {
                let dt = dep.to_string_lossy();
                ser.push_str(&format!("+{dt}\n"));
            }
        }
        for (key, entry) in &sourcemap {
            let pt = key.to_string_lossy();
            ser.push_str(&format!(
                ">{pt}\x1f{}\x1f{}\n",
                entry.content, entry.modified
            ));
        }

        fs::write(build_dir.join("cedar.d"), ser.as_bytes())?;
        Ok(Self {
            headers: headermap,
            sources: sourcemap,
            dir: path.to_path_buf(),
        })
    }
    pub fn write(&mut self) -> io::Result<()> {
        let build_dir = self.dir.join("build/");

        let mut ser = String::new();

        for (key, entry) in &self.headers {
            let pt = key.to_string_lossy();
            ser.push_str(&format!(
                "<{pt}\x1f{}\x1f{}\n",
                entry.content, entry.modified
            ));
            for dep in &entry.dependents {
                let dt = dep.to_string_lossy();
                ser.push_str(&format!("+{dt}\n"));
            }
        }
        for (key, entry) in &self.sources {
            let pt = key.to_string_lossy();
            ser.push_str(&format!(
                ">{pt}\x1f{}\x1f{}\n",
                entry.content, entry.modified
            ));
        }

        fs::write(build_dir.join("cedar.d"), ser.as_bytes())?;
        Ok(())
    }
    pub fn read(&mut self) -> io::Result<()> {
        let dpath = self.dir.join("build/cedar.d");
        println!("READ: {dpath:?}");
        let bytes = fs::read(dpath)?;

        println!("READ");

        let mut xptr = bytes.as_ptr();
        let mut yptr = xptr;
        let mut xrdr: u8;
        let mut yrdr;
        let end = unsafe { xptr.add(bytes.len()) };
        let mut last_header = Rc::<Path>::from(PathBuf::from("").as_ref());
        while let Some((newline, _)) = findbyte(xptr, b'\n', end) {
            unsafe {
                xptr = newline;
                yrdr = *yptr;
                match yrdr {
                    b'<' => {
                        let path = if let Some((pend, plen)) = findbyte(yptr, b'\x1f', end) {
                            let path = str::from_utf8_unchecked(std::slice::from_raw_parts(
                                yptr.add(1),
                                plen - 1,
                            ));
                            yptr = pend.add(1);
                            Rc::<Path>::from(PathBuf::from(path).as_ref())
                        } else {
                            unreachable!()
                        };
                        let hash = if let Some((hend, hlen)) = findbyte(yptr, b'\x1f', end) {
                            let hash =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, hlen));
                            yptr = hend.add(1);
                            hash.parse::<u64>().unwrap()
                        } else {
                            unreachable!()
                        };
                        let modify = if let Some((mend, mlen)) = findbyte(yptr, b'\n', end) {
                            let modify =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, mlen));
                            modify.parse::<u64>().unwrap()
                        } else {
                            unreachable!()
                        };

                        last_header = path.clone();

                        self.headers.insert(
                            path.clone(),
                            CHeader {
                                path,
                                content: hash,
                                modified: modify,
                                dependents: Vec::new(),
                            },
                        );
                    }
                    b'+' => {
                        let slice = std::slice::from_raw_parts(
                            yptr.add(1),
                            xptr as usize - yptr as usize - 1,
                        );
                        let str = str::from_utf8_unchecked(slice);
                        if let Some(h) = self.headers.get_mut(&last_header) {
                            h.dependents.push(Rc::<Path>::from(PathBuf::from(str)));
                        }
                    }
                    b'>' => {
                        let path = if let Some((pend, plen)) = findbyte(yptr, b'\x1f', end) {
                            let path = str::from_utf8_unchecked(std::slice::from_raw_parts(
                                yptr.add(1),
                                plen - 1,
                            ));
                            yptr = pend.add(1);
                            Rc::<Path>::from(PathBuf::from(path).as_ref())
                        } else {
                            unreachable!()
                        };
                        let hash = if let Some((hend, hlen)) = findbyte(yptr, b'\x1f', end) {
                            let hash =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, hlen));
                            yptr = hend.add(1);
                            hash.parse::<u64>().unwrap()
                        } else {
                            unreachable!()
                        };
                        let modify = if let Some((mend, mlen)) = findbyte(yptr, b'\n', end) {
                            let modify =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, mlen));
                            modify.parse::<u64>().unwrap()
                        } else {
                            unreachable!()
                        };
                        self.sources.insert(
                            path.clone(),
                            CSource {
                                path,
                                modified: modify,
                                content: hash,
                            },
                        );
                    }
                    _ => {}
                }
                yptr = xptr.add(1);
                xptr = xptr.add(1);
            }
        }

        Ok(())
    }
    pub fn to_compile(&mut self) -> io::Result<Vec<Rc<Path>>> {
        println!("to_compile");
        // Ensure it is synced to the cedar.d file.
        self.read()?;
        let mut out = Vec::new();
        for srcfile in walk_dir(&self.dir)? {
            println!("{srcfile:?}");
            let rc = Rc::<Path>::from(srcfile.as_ref());
            let modify = &rc
                .metadata()?
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if srcfile.extension().is_some_and(|fs| fs == "c") {
                self.sources
                    .entry(rc.clone())
                    .and_modify(|src| {
                        if src.modified != *modify {
                            let bytes = fs::read(&rc).unwrap();
                            let hash = fxhash(&bytes);
                            src.modified = *modify;
                            if src.content != hash {
                                src.content = hash;
                                out.push(rc.clone());
                            }
                        }
                    })
                    .or_insert_with(|| {
                        out.push(rc.clone());
                        CSource {
                            modified: *modify,
                            path: rc.clone(),
                            content: fxhash(&fs::read(rc).unwrap()),
                        }
                    });
            } else if srcfile.extension().is_some_and(|ex| ex == "h") {
                self.headers
                    .entry(rc.clone())
                    .and_modify(|head| {
                        if head.modified != *modify {
                            let bytes = fs::read(&rc).unwrap();
                            let hash = fxhash(&bytes);
                            head.modified = *modify;
                            if head.content != hash {
                                head.content = hash;
                                for dep in &head.dependents {
                                    out.push(dep.clone());
                                }
                            }
                        }
                    })
                    .or_insert_with(|| {
                        out.push(rc.clone());
                        CHeader {
                            modified: *modify,
                            path: rc.clone(),
                            content: fxhash(&fs::read(rc).unwrap()),
                            dependents: Vec::new(),
                        }
                    });
            }
        }
        println!("OUT:: {out:?}");
        Ok(out)
    }
}

#[cfg(test)]
mod core_dag_t {
    use crate::core::dag::DependencyGraph;

    #[test]
    fn init_t() {
        let mut dag = DependencyGraph::new("./tests/data/calc/");
        dag.read().unwrap();
        let to_compile = dag.to_compile().unwrap();
    }
}
