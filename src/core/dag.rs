use std::{
    ffi::c_void,
    fs, io,
    path::{Path, PathBuf},
};

use ahash::AHashMap;

unsafe extern "C" {
    pub(crate) fn memchr(ptr: *const c_void, c: i32, len: usize) -> *const c_void;
}

#[inline(always)]
pub fn _memchr(ptr: *const u8, c: u8, len: usize) -> *const u8 {
    unsafe { memchr(ptr as *const c_void, c as i32, len) as *const u8 }
}

pub struct TargetNode<'a> {
    name: &'a str,
    id: usize,
    hash: u64,
    depends: Vec<(&'a str, usize)>,
}

pub struct DependGraph<'a> {
    nodes: Vec<TargetNode<'a>>,
    idmap: AHashMap<&'a str, usize>,
}

const WORD_SIZE: usize = (usize::BITS / 8) as usize;
const HAS_ZERO_SUB: usize = (0x0101_0101_0101_0101u64) as usize;
const HAS_ZERO_AND: usize = (0x8080_8080_8080_8080u64) as usize;
const QUOTE: usize = 0x2222222222222222;

const BRACKET: usize = 0x3c3c3c3c3c3c3c3c;
const NEWLINE: usize = 0x0a0a0a0a0a0a0a0a;
const BACKSLASH: usize = 0x5c5c5c5c5c5c5c5c;

impl<'a> DependGraph<'a> {
    pub fn build<P: AsRef<Path>>(path: P) -> Self {
        let mut nodes = Vec::new();
        let mut idmap = AHashMap::new();
        let mut deps = AHashMap::new();

        fn rec<P: AsRef<Path>>(nds: &mut AHashMap<String, Vec<String>>, p: P) -> io::Result<()> {
            let path = p.as_ref();
            for dir in path.read_dir()?.flatten() {
                if dir.file_type()?.is_dir() {
                    rec(nds, dir.path())?;
                } else if dir.file_type()?.is_file() {
                    let path = dir.path();
                    if let Some(name) = path.file_name().map(|name| name.to_string_lossy())
                        && name.ends_with(['c', 'h'])
                    {
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
                                                let include = str::from_utf8_unchecked(
                                                    std::slice::from_raw_parts(start, len),
                                                )
                                                .to_string();
                                                nds.entry(name.to_string())
                                                    .and_modify(|v| v.push(include.clone()))
                                                    .or_insert(vec![include]);
                                                state = 0;
                                                break;
                                            }
                                            ptr = ptr.add(1);
                                        }
                                    }
                                    _ => {}
                                }
                                ptr = ptr.add(1)
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        rec(&mut deps, path).unwrap();

        Self { nodes, idmap }
    }
}

#[inline(always)]
const fn has_zero(x: usize) -> usize {
    (x.wrapping_sub(HAS_ZERO_SUB)) & !x & HAS_ZERO_AND
}
#[cfg(test)]
mod core_dag_t {
    use crate::core::dag::{DependGraph, has_zero};

    #[test]
    fn init_t() {
        let dag = DependGraph::build("/home/june/archive/repo/gcc/");

        //let dag = DependGraph::build("./tests/data/calc/");
    }
}
