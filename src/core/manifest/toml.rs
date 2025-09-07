use std::{collections::HashMap, path::Path, process::exit, rc::Rc, string::ParseError};

use crate::core::{error::ManifestError, utils::findbyte};

/// A representation of the TOML types relevant to Cedar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    List(Vec<Value>),
    Inline(Vec<(Value, Value)>),
}

impl Value {
    /// Parse a TOML value into a Value enum.
    ///
    /// Only the relevant TOML types are differentiated. So, numbers will be
    /// treated as strings. A list must start with and end with brackets, and
    /// this function assumes the source has been stripped.
    ///
    /// # Returns
    ///
    /// This function is not fallible, so, in cases where invalid data is
    /// passed in, a String containing the data will be returned.
    pub fn parse(source: &str) -> Self {
        if source.starts_with('[') && source.ends_with(']') {
            if source.len() < 3 {
                return Value::List(Vec::new());
            }
            let trimmed = source
                .trim_start_matches("[")
                .trim_end_matches("]")
                .split(',')
                .filter_map(|mem| {
                    if !mem.is_empty() {
                        Some(Value::parse(mem.trim().trim_matches('"')))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            Self::List(trimmed)
        } else if source.starts_with('{') && source.ends_with('}') {
            todo!()
        } else {
            Self::String(source.trim_matches('"').to_string())
        }
    }
}

/// A representation of a single table TOML block.
#[derive(Debug)]
pub struct Table {
    pub name: Rc<str>,
    kv: HashMap<Rc<str>, Value>,
}

impl Table {
    /// Parses a single table block from a TOML file.
    ///
    /// # Arguments
    ///
    /// * 'ptr' - A pointer to the start of the block file, when dereferenced
    ///   this should equal b'[''.
    /// * 'end' - The end of the table block, not the end of the byte buffer.
    pub fn parse(ptr: *const u8, end: *const u8) -> Result<Self, ManifestError> {
        // Main pointer.
        let mut xptr = tfindbyte(ptr, b'[', end)?;
        // Searching pointer, for backtracking/scouting.
        let mut yptr;
        // Holders for their corronsponding pointers dereferenced value.
        let mut xrdr;
        let mut yrdr;

        let name_end = tfindbyte(xptr, b']', end)?;
        let name_len = name_end as usize - xptr as usize;

        let name = match str::from_utf8(unsafe {
            std::slice::from_raw_parts(xptr.add(1), name_len - 1)
        }) {
            Ok(n) => Rc::<str>::from(n),
            Err(_) => return Err(ManifestError::ParseError(ptr, xptr, end)),
        };

        let mut kv = HashMap::new();

        xptr = name_end;
        while xptr < end {
            unsafe {
                // On the first iteration:
                // [example] <---- xptr is here.
                // key = "value"
                //    ^----------- yptr is here.
                //
                // On the second:
                // [example]
                // key = "value" <---- xptr is here.
                // key2 = "value2"
                //     ^----------- yptr is here.
                xptr = match tfindbyte(xptr, b'\n', end) {
                    Ok(p) => p,
                    Err(_) => break,
                };
                yptr = match tfindbyte(xptr, b'=', end) {
                    Ok(p) => p.sub(1),
                    Err(_) => break,
                };

                // Trim whitespace.
                // On the first iteration:
                // [example] <---- xptr is here.
                //     \r\r\n\n\t\t evil_key     = "evil_value"
                //             yptr is here ----^
                //
                // The pointers then squish towards the key as long as the
                // byte is < 32, which means ASCII whitespace and control
                // bytes.
                let key_len = loop {
                    xrdr = *xptr;
                    yrdr = *yptr;
                    if yptr <= xptr {
                        return Err(ManifestError::ParseError(ptr, xptr, end));
                    }
                    match (xrdr > 32, yrdr > 32) {
                        (true, true) => break yptr.add(1) as usize - xptr as usize,
                        (false, true) => xptr = xptr.add(1),
                        (true, false) => yptr = yptr.sub(1),
                        (false, false) => {
                            xptr = xptr.add(1);
                            yptr = yptr.sub(1);
                        }
                    }
                };

                let key = Rc::<str>::from(str::from_utf8_unchecked(std::slice::from_raw_parts(
                    xptr, key_len,
                )));

                // [example]
                // key = "value" <- yptr
                //     ^- xptr
                xptr = tfindbyte(yptr, b'=', end)?.add(1);
                yptr = tfindbyte(yptr, b'\n', end)?;

                if *xptr.add(1) == b'[' {
                    yptr = tfindbyte(xptr, b']', end)?.add(1);
                    let val_str = str::from_utf8_unchecked(std::slice::from_raw_parts(
                        xptr,
                        yptr.offset_from_unsigned(xptr),
                    ));
                    let parsed = val_str.lines().map(|x| x.trim()).collect::<String>();
                    let val = Value::parse(&parsed);
                    kv.insert(key, val);

                    xptr = yptr;
                } else {
                    // Trim whitespace again.
                    let val_len = loop {
                        xrdr = *xptr;
                        yrdr = *yptr;
                        if yptr < xptr {
                            println!("err");
                            return Err(ManifestError::ParseError(ptr, xptr, end));
                        }
                        match (xrdr > 32, yrdr > 32) {
                            (true, true) => break yptr.add(1) as usize - xptr as usize,
                            (false, true) => xptr = xptr.add(1),
                            (true, false) => yptr = yptr.sub(1),
                            (false, false) => {
                                xptr = xptr.add(1);
                                yptr = yptr.sub(1);
                            }
                        }
                    };

                    let val = Value::parse(str::from_utf8_unchecked(std::slice::from_raw_parts(
                        xptr, val_len,
                    )));

                    kv.insert(key, val);
                }
            }
        }

        Ok(Self { name, kv })
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.kv.get(key)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Rc<str>, &Value)> {
        self.kv.iter()
    }
}

fn tfindbyte(ptr: *const u8, c: u8, end: *const u8) -> Result<*const u8, ManifestError> {
    match findbyte(ptr, c, end) {
        Ok(p) => Ok(p),
        Err(eptr) => Err(ManifestError::ParseError(ptr, eptr, end)),
    }
}

pub fn toml_parse(bytes: &[u8]) -> Vec<Table> {
    let mut ptr = bytes.as_ptr();
    let end = unsafe { ptr.add(bytes.len()) };
    let mut tables = Vec::new();
    let mut section_pointers = Vec::new();
    unsafe {
        while ptr < end {
            let byte = *ptr;
            if byte == b'=' {
                ptr = match tfindbyte(ptr, b'\n', end) {
                    Ok(p) => p,
                    Err(_) => break,
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
                Ok(t) => tables.push(t),
                Err(e) => {
                    eprintln!("{e}");
                    exit(1);
                }
            };
        }
    }

    tables
}

pub fn toml_parse_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Table>> {
    let bytes = std::fs::read(path)?;
    Ok(toml_parse(&bytes))
}

#[cfg(test)]
mod core_manifest_toml_t {
    use core::panic;
    use std::{error::Error, fs};

    use crate::core::manifest::{
        EXAMPLE_MANIFEST,
        toml::{Table, Value, toml_parse},
    };

    #[test]
    fn parse_table_t() {
        let table = "[example]\nkind_key = \"kind_value\"\n\r\t\t\r\t\t\r   evil_key = \"kind_value\"\n  \t\r\t\r\t   also_evil_key          =            \"evil_value\"      \n ";
        let ptr = table.as_ptr();
        let end = unsafe { ptr.add(table.len()) };
        let parsed = Table::parse(ptr, end).unwrap();

        assert_eq!(
            parsed.kv.get("kind_key"),
            Some(crate::core::manifest::toml::Value::String(
                "kind_value".to_string()
            ))
            .as_ref()
        );
        assert_eq!(
            parsed.kv.get("evil_key"),
            Some(crate::core::manifest::toml::Value::String(
                "kind_value".to_string()
            ))
            .as_ref()
        );
        assert_eq!(
            parsed.kv.get("also_evil_key"),
            Some(crate::core::manifest::toml::Value::String(
                "evil_value".to_string()
            ))
            .as_ref()
        );
    }

    #[test]
    fn parse_multi_line_t() {
        let table = fs::read("/home/june/repo/malloc/cedar.toml").unwrap();
        let parsed = toml_parse(&table);
        println!("{parsed:?}");
    }

    #[test]
    fn parse_full_t() {
        let manifest = EXAMPLE_MANIFEST.as_bytes();
        let parse = toml_parse(manifest);
        let meta = &parse[0];
        let build = &parse[1];
        let deps = &parse[2];
        let name = meta.get("name").unwrap();
        let version = meta.get("version").unwrap();
        let compiler = build.get("compiler").unwrap();
        let cflags = build.get("cflags").unwrap();

        assert_eq!(name, &Value::String("main".to_string()));
        assert_eq!(version, &Value::String("0.1.0".to_string()));
        assert_eq!(compiler, &Value::String("clang".to_string()));
        assert_eq!(
            cflags,
            &Value::List(vec![
                Value::String("-Wall".to_string()),
                Value::String("-Wextra".to_string())
            ])
        );
    }
}
