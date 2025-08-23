//! TOML parser for only the features needed by cedar.

use std::{path::Path, rc::Rc};

use ahash::AHashMap;

use crate::core::utils::{findbyte, memchr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    List(Vec<Value>),
    Inline(Vec<(Value, Value)>),
}

impl Value {
    pub fn parse(source: &str) -> Self {
        if source.starts_with('[') && source.ends_with(']') {
            // TODO: Single pass linear extract val.
            let trimmed = source
                .trim_start_matches("[\"")
                .trim_end_matches("\"]")
                .split(',')
                .map(|mem| Value::parse(mem.trim().trim_matches('"')))
                .collect::<Vec<_>>();

            Self::List(trimmed)
        } else if source.starts_with('{') && source.ends_with('}') {
            todo!()
        } else {
            Self::String(source.trim_matches('"').to_string())
        }
    }
}

#[derive(Debug)]
pub struct Table {
    name: String,
    kv: AHashMap<String, Value>,
}

impl Table {
    /// Parse from a pointer to the start section to the given end pointer.
    /// If the table header is '[meta]\n', this assumes the pointer is at the
    /// '['.
    pub fn parse(ptr: *const u8, end: *const u8) -> Result<Self, *const u8> {
        // Main pointer.
        let (mut xptr, _) = match findbyte(ptr, b'[', end) {
            Some((p, j)) => (p, j),
            None => return Err(ptr),
        };
        // Searching pointer, for backtracking/scouting.
        let mut yptr;
        let mut xrdr;
        let mut yrdr;
        let mut kv = AHashMap::new();

        let (name_end, name_len) = match findbyte(xptr, b']', end) {
            Some((p, j)) => (p, j),
            None => return Err(xptr),
        };

        let name = match str::from_utf8(unsafe {
            std::slice::from_raw_parts(xptr.add(1), name_len - 1)
        }) {
            Ok(n) => n.to_string(),
            Err(_) => return Err(xptr),
        };

        xptr = name_end;
        while xptr < end {
            unsafe {
                if let Some((nxt, _)) = findbyte(xptr, b'\n', end) {
                    xptr = nxt;
                    if let Some((eq, eqpos)) = findbyte(nxt, b'=', end) {
                        yptr = eq.sub(1);

                        // Trim whitespace.
                        let len = loop {
                            yrdr = *yptr;
                            xrdr = *xptr;
                            if yptr <= xptr {
                                return Err(xptr);
                            }
                            match (xrdr > 32, yrdr > 32) {
                                (true, true) => break yptr.add(1) as usize - xptr as usize,
                                (false, true) => xptr = xptr.add(1),
                                (true, false) => yptr = yptr.sub(1),
                                _ => {
                                    xptr = xptr.add(1);
                                    yptr = yptr.sub(1);
                                }
                            }
                        };

                        let key = str::from_utf8_unchecked(std::slice::from_raw_parts(xptr, len))
                            .to_string();

                        xptr = eq.add(1);
                        yptr = findbyte(xptr, b'\n', end).map(|x| x.0).unwrap_or(end);

                        let val_len = loop {
                            xrdr = *xptr;
                            yrdr = *yptr;
                            if yptr <= xptr {
                                return Err(xptr);
                            }
                            match (xrdr > 32, yrdr > 32) {
                                (true, true) => break yptr.add(1) as usize - xptr as usize,
                                (false, true) => xptr = xptr.add(1),
                                (true, false) => yptr = yptr.sub(1),
                                _ => {
                                    xptr = xptr.add(1);
                                    yptr = yptr.sub(1);
                                }
                            }
                        };

                        let raw_val =
                            str::from_utf8_unchecked(std::slice::from_raw_parts(xptr, val_len));

                        let val = Value::parse(raw_val);

                        kv.insert(key, val);
                    }
                }
                xptr = xptr.add(1)
            };
        }

        Ok(Self { name, kv })
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.kv.get(key)
    }
}

pub fn toml_parse(bytes: &[u8]) -> AHashMap<Rc<str>, Table> {
    let mut ptr = bytes.as_ptr();
    let end = unsafe { ptr.add(bytes.len()) };
    let mut tables = AHashMap::new();
    let mut section_pointers = Vec::new();
    unsafe {
        while ptr < end {
            let byte = *ptr;
            if byte == b'=' {
                ptr = match findbyte(ptr, b'\n', end) {
                    Some((p, _)) => p,
                    None => break,
                };
                continue;
            }
            if byte == b'[' {
                section_pointers.push(ptr);
            }
            ptr = ptr.add(1);
        }
    }
    section_pointers.push(end);

    for window in section_pointers.windows(2) {
        if window.len() == 2 {
            match Table::parse(window[0], window[1]) {
                Ok(t) => tables.insert(Rc::from(t.name.as_ref()), t),
                Err(_) => todo!(),
            };
        }
    }

    tables
}

pub fn toml_parse_file<P: AsRef<Path>>(path: P) -> std::io::Result<AHashMap<Rc<str>, Table>> {
    let bytes = std::fs::read(path)?;
    Ok(toml_parse(&bytes))
}

#[cfg(test)]
mod core_toml_t {
    use crate::core::{
        manifest::EXAMPLE_MANIFEST,
        toml::{Table, toml_parse},
    };

    #[test]
    fn table_parse_t() {
        let clean = r#"[meta]
name = "main"
version = "0.1.0""#;
        let weird = r#"   [meta]
     name     =    "main"   
  version="0.1.0"
"#;
        let (cptr, wptr) = (clean.as_ptr(), weird.as_ptr());
        let clean_res = Table::parse(cptr, unsafe { cptr.add(clean.len()) }).unwrap();
        let weird_res = Table::parse(wptr, unsafe { wptr.add(weird.len()) }).unwrap();

        assert_eq!(clean_res.get("name"), weird_res.get("name"));
        assert_eq!(clean_res.get("version"), weird_res.get("version"));
    }

    #[test]
    fn file_parse_t() {
        let example = EXAMPLE_MANIFEST.as_bytes();
        let file = toml_parse(example);
        let meta = file.get("meta");
        let build = file.get("build");
        let deps = file.get("dependencies");
        assert!(meta.is_some());
        assert!(build.is_some());
        assert!(deps.is_some());
    }
}
