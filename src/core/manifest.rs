use ahash::AHashMap;
use std::{collections::HashMap, fmt::Debug, path::Path, rc::Rc};

use crate::core::{
    builder::Compiler,
    error::{BuilderError, ManifestError},
    toml::{Value, toml_parse},
};

pub const EXAMPLE_MANIFEST: &str = r#"
[meta]
name = "main"
version = "0.1.0"

[build]
compiler = "clang"
cflags = ["-Wall", "-Wextra"]

[dependencies]
todo = "include/todo"
"#;

pub struct Manifest {
    raw: Rc<str>,
    pub name: Rc<str>,
    pub version: Option<Rc<str>>,
    compiler: Rc<str>,
    cflags: Vec<Rc<str>>,
    deps: Option<AHashMap<Rc<str>, Rc<Path>>>,
}

impl Manifest {
    pub fn parse(src: &str) -> Result<Self, ManifestError> {
        let parsed = toml_parse(src.as_bytes());
        let (name, version) = match parsed.get("meta").map(|meta| {
            (
                meta.get("name").and_then(|val| {
                    if let Value::String(x) = val {
                        Some::<Rc<str>>(Rc::from(x.as_str()))
                    } else {
                        None
                    }
                }),
                meta.get("version").and_then(|val| {
                    if let Value::String(x) = val {
                        Some(Rc::from(x.as_str()))
                    } else {
                        None
                    }
                }),
            )
        }) {
            Some(val) => val,
            None => return Err(ManifestError::Invalid),
        };

        let (compiler, cflags): (Option<Rc<str>>, _) = match parsed.get("build") {
            Some(table) => (
                table.get("compiler").and_then(|val| {
                    if let Value::String(x) = val {
                        Some(Rc::from(x.as_str()))
                    } else {
                        None
                    }
                }),
                table
                    .get("cflags")
                    .and_then(|val| {
                        if let Value::List(x) = val {
                            Some(
                                x.iter()
                                    .filter_map(|str| {
                                        if let Value::String(s) = str {
                                            Some(Rc::from(s.as_str()))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            )
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Vec::new()),
            ),
            None => return Err(ManifestError::Invalid),
        };

        if name.is_none() {
            Err(ManifestError::MissingName)
        } else if compiler.is_none() {
            Err(ManifestError::MissingCompiler)
        } else if let (Some(n), v, Some(c)) = (name, version, compiler) {
            Ok(Self {
                raw: Rc::from(src),
                name: n.clone(),
                version: v.clone(),
                compiler: c.clone(),
                cflags,
                deps: None,
            })
        } else {
            Err(ManifestError::Invalid)
        }
    }
    pub fn compiler(&self) -> Result<Compiler, BuilderError> {
        Compiler::build(self.compiler.as_ref(), &self.cflags)
    }
}

impl Debug for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.raw)?;
        writeln!(
            f,
            "name: {}, {}compiler: {}, flags: {:?}",
            self.name,
            if let Some(v) = &self.version {
                format!("version: {v}, ")
            } else {
                "".to_string()
            },
            self.compiler,
            self.cflags
        )
    }
}

#[cfg(test)]
mod core_manifest_t {
    use std::rc::Rc;

    use crate::core::manifest::{EXAMPLE_MANIFEST, Manifest};

    #[test]
    fn parse_t() {
        let mani = Manifest::parse(EXAMPLE_MANIFEST).unwrap();
        assert_eq!(mani.name.as_ref(), "main");
        assert_eq!(mani.version, Some(Rc::from("0.1.0")));
        assert_eq!(mani.compiler.as_ref(), "clang");
        assert_eq!(mani.cflags, vec![Rc::from("-Wall"), Rc::from("-Wextra")]);
    }
}
