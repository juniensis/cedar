use std::{
    fs, io,
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

pub struct DependencyGraph {
    sources: Vec<CSource>,
    headers: Vec<CHeader>,
    dir: PathBuf,
}

impl DependencyGraph {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
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

        let mut headers = Vec::new();
        let mut sources = Vec::new();
        let mut ser = String::new();

        for (key, entry) in headermap {
            let pt = key.to_string_lossy();
            ser.push_str(&format!(
                "<{pt}\x1f{}\x1f{}\n",
                entry.content, entry.modified
            ));
            for dep in &entry.dependents {
                let dt = dep.to_string_lossy();
                ser.push_str(&format!("+{dt}\n"));
            }
            headers.push(entry)
        }
        for (key, entry) in sourcemap {
            let pt = key.to_string_lossy();
            ser.push_str(&format!(
                ">{pt}\x1f{}\x1f{}\n",
                entry.content, entry.modified
            ));
            sources.push(entry)
        }

        fs::write(build_dir.join("cedar.d"), ser.as_bytes())?;
        Ok(Self {
            headers,
            sources,
            dir: path.to_path_buf(),
        })
    }
    pub fn read<P: AsRef<Path>>(dpath: P) -> io::Result<Self> {
        let bytes = fs::read(dpath)?;

        let mut xptr = bytes.as_ptr();
        let mut yptr = xptr;
        let mut xrdr: u8;
        let mut yrdr;
        let end = unsafe { xptr.add(bytes.len()) };

        //let mut headers = Vec::new();
        //let mut source = Vec::new();

        while let Some((newline, _)) = findbyte(xptr, b'\n', end) {
            unsafe {
                xptr = newline;
                yrdr = *yptr;
                match yrdr {
                    b'<' => {
                        // Right now, xptr is at the end of the line, and yptr
                        // is at the start of the line.
                        println!("HEADER:");

                        let path = if let Some((pend, plen)) = findbyte(yptr, b'\x1f', end) {
                            let path = str::from_utf8_unchecked(std::slice::from_raw_parts(
                                yptr.add(1),
                                plen - 1,
                            ));
                            yptr = pend.add(1);
                            Rc::<Path>::from(PathBuf::from(path).as_ref())
                        } else {
                            todo!()
                        };
                        let hash = if let Some((hend, hlen)) = findbyte(yptr, b'\x1f', end) {
                            let hsh =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, hlen));
                            yptr = hend.add(1);
                            hsh.parse::<u64>().unwrap()
                        } else {
                            todo!()
                        };
                        let modify = if let Some((mend, mlen)) = findbyte(yptr, b'\n', end) {
                            let mdfy =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, mlen));
                            mdfy.parse::<u64>().unwrap()
                        } else {
                            todo!()
                        };
                    }
                    b'+' => {
                        let slice = std::slice::from_raw_parts(
                            yptr.add(1),
                            xptr as usize - yptr as usize - 1,
                        );
                        let str = str::from_utf8_unchecked(slice);
                        println!("DEPENDANT: {str}");
                    }
                    b'>' => {
                        println!("SOURCE:");
                        if let Some((pend, plen)) = findbyte(yptr, b'\x1f', end) {
                            let path = str::from_utf8_unchecked(std::slice::from_raw_parts(
                                yptr.add(1),
                                plen - 1,
                            ));
                            yptr = pend.add(1);
                            println!("PATH: {path}");
                        }
                        if let Some((hend, hlen)) = findbyte(yptr, b'\x1f', end) {
                            let hash =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, hlen));
                            yptr = hend.add(1);
                            println!("HASH: {hash}");
                        }
                        if let Some((mend, mlen)) = findbyte(yptr, b'\n', end) {
                            let modify =
                                str::from_utf8_unchecked(std::slice::from_raw_parts(yptr, mlen));
                            println!("MOD: {modify}");
                        }
                    }
                    _ => {}
                }
                yptr = xptr.add(1);
                xptr = xptr.add(1);
            }
        }

        todo!()
    }
}

#[cfg(test)]
mod core_dag_t {
    use crate::core::dag::DependencyGraph;

    #[test]
    fn init_t() {
        let dag = DependencyGraph::new("./tests/data/calc/").unwrap();
        let de = DependencyGraph::read("./tests/data/calc/build/cedar.d").unwrap();
    }
}
