use std::{
    ffi::c_void,
    fmt::{Debug, Display},
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use ahash::AHashMap;

use crate::core::utils::memchr;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CFile {
    Header { name: Rc<str>, path: Rc<Path> },
    Source { name: Rc<str>, path: Rc<Path> },
}

pub struct TargetNode {
    data: CFile,
    id: usize,
    depends: Vec<usize>,
}

pub struct DependGraph {
    edges: AHashMap<CFile, Vec<CFile>>,
}

impl DependGraph {
    pub fn build<P: AsRef<Path>>(path: P) -> Self {
        let mut edges = AHashMap::new();

        // Map file -> dependencies.
        fn rec<P: AsRef<Path>>(out: &mut AHashMap<CFile, Vec<CFile>>, p: P) -> io::Result<()> {
            let path = p.as_ref();
            for dir in path.read_dir()?.flatten() {
                if dir.file_type()?.is_dir() {
                    rec(out, dir.path())?;
                } else if dir.file_type()?.is_file() {
                    let path = dir.path();
                    if let Some(name) = path.file_name().map(|name| name.to_string_lossy())
                        && (name.ends_with(".c") || name.ends_with(".h"))
                    {
                        extract_includes(Rc::from(name), path, out)?;
                    }
                }
            }
            Ok(())
        }

        rec(&mut edges, path).unwrap();

        // Flip map to dependency -> depended on by.
        let mut flipped: AHashMap<CFile, Vec<CFile>> = AHashMap::new();

        for (dependent, dependencies) in edges.into_iter() {
            for dependency in dependencies {
                flipped
                    .entry(dependency)
                    .and_modify(|v| v.push(dependent.clone()))
                    .or_insert(vec![dependent.clone()]);
            }
        }

        Self { edges: flipped }
    }
}

impl Display for CFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Header { name, path } => write!(f, "{name}"),
            Self::Source { name, path } => write!(f, "{name}"),
        }
    }
}

impl Debug for DependGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.edges.iter() {
            writeln!(f, "{k}:")?;
            for dep in v {
                writeln!(f, "  -> {dep}")?;
            }
        }
        Ok(())
    }
}

// +-----------------+
// | Include parsing |
// +-----------------+

#[inline(always)]
pub fn _memchr(ptr: *const u8, c: u8, len: usize) -> *const u8 {
    unsafe { memchr(ptr as *const c_void, c as i32, len) as *const u8 }
}

fn extract_includes<P: AsRef<Path>>(
    name: Rc<str>,
    path: P,
    out: &mut AHashMap<CFile, Vec<CFile>>,
) -> io::Result<()> {
    let buf = fs::read(&path)?;
    let mut ptr = buf.as_ptr();
    let bound = unsafe { ptr.add(buf.len()) };
    let mut start = ptr;
    let mut state = 0;
    unsafe {
        while ptr < bound {
            match state {
                0 => {
                    ptr = _memchr(ptr, b'#', bound as usize - ptr as usize);
                    if ptr.is_null() {
                        break;
                    }
                    if (bound as usize - ptr as usize) > 8
                        && *(ptr as *const [u8; 8]) == *b"#include"
                    {
                        ptr = ptr.add(8);
                        state = 1;
                    }
                }
                1 => {
                    while ptr < bound {
                        let byte = *ptr;
                        if byte == b'\\' {
                            ptr = ptr.add(2);
                            continue;
                        } else if byte == b'\n' {
                            state = 0;
                            break;
                        } else if byte == b'"' {
                            ptr = ptr.add(1);
                            start = ptr;
                            state = 2;
                            break;
                        }
                        ptr = ptr.add(1);
                    }
                }
                2 => {
                    while ptr < bound {
                        let byte = *ptr;
                        if byte == b'\\' {
                            ptr = ptr.add(2);
                            continue;
                        } else if byte == b'\n' {
                            state = 0;
                            break;
                        } else if byte == b'"' {
                            let len = ptr as usize - start as usize;
                            let include: Rc<str> = Rc::from(str::from_utf8_unchecked(
                                std::slice::from_raw_parts(start, len),
                            ));

                            let include_path = match resolve_header_path(
                                path.as_ref(),
                                include.as_ref(),
                            ) {
                                Ok(p) => p,
                                Err(p) => {
                                    eprintln!(
                                        "Warning: invalid include statement in {name}.\n Cannot find {p:?}"
                                    );
                                    state = 0;
                                    break;
                                }
                            };

                            let include_file = CFile::Header {
                                name: Rc::from(include.split("/").last().unwrap()),
                                path: Rc::from(include_path.as_ref()),
                            };
                            out.entry(if name.ends_with(".c") {
                                CFile::Source {
                                    name: name.clone(),
                                    path: Rc::from(path.as_ref()),
                                }
                            } else {
                                CFile::Header {
                                    name: name.clone(),
                                    path: Rc::from(path.as_ref()),
                                }
                            })
                            .and_modify(|v| v.push(include_file.clone()))
                            .or_insert(vec![include_file]);
                            state = 0;
                            break;
                        }
                        ptr = ptr.add(1);
                    }
                }
                _ => state = 0,
            }
            ptr = ptr.add(1)
        }
    }

    Ok(())
}

fn resolve_header_path<P: AsRef<Path>>(
    source_path: P,
    include_statement: &str,
) -> Result<PathBuf, &str> {
    let components = include_statement.split("/").collect::<Vec<_>>();
    let source_parent = match source_path.as_ref().parent() {
        Some(p) => p.to_path_buf(),
        None => return Err(include_statement),
    };

    if components.len() == 1 || components[0] == "." {
        match fs::canonicalize(source_parent.join(include_statement)) {
            Ok(p) => Ok(p),
            Err(_) => Err(include_statement),
        }
    } else if components[0] == ".." {
        let mut adjusted = source_parent.clone();
        for (par, &comp) in source_parent.ancestors().zip(components.iter()) {
            if comp == ".." {
                adjusted = par.parent().unwrap().to_path_buf();
            } else {
                adjusted = adjusted.join(comp);
                break;
            }
        }
        match fs::canonicalize(&adjusted) {
            Ok(p) => Ok(p),
            Err(_) => Err(include_statement),
        }
    } else if let Some(parent) = source_path.as_ref().parent() {
        match fs::canonicalize(parent.join(include_statement)) {
            Ok(p) => Ok(p),
            Err(_) => Err(include_statement),
        }
    } else {
        Err(include_statement)
    }
}

#[cfg(test)]
mod core_dag_t {
    use crate::core::dag::DependGraph;

    #[test]
    fn init_t() {
        //let dag = DependGraph::build("/home/june/archive/repo/gcc/");

        let dag = DependGraph::build("./tests/data/calc/");
    }
}
