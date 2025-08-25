use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::core::{
    error::ManifestError,
    manifest::toml::{Value, toml_parse},
};

pub mod toml;

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

#[derive(Debug)]
pub struct Manifest {
    raw: Rc<str>,
    pub name: Rc<str>,
    pub version: Option<Rc<str>>,
    compiler: Rc<str>,
    cflags: Vec<Rc<str>>,
    deps: Option<HashMap<Rc<str>, PathBuf>>,
}

impl Manifest {
    pub fn parse_str(src: &str) -> Result<Self, ManifestError> {
        let tables = toml_parse(src.as_bytes());
        let mut name: Rc<str> = Default::default();
        let mut version: Option<Rc<str>> = Default::default();
        let mut compiler: Rc<str> = Default::default();
        let mut cflags: Vec<Rc<str>> = Vec::new();
        let mut deps: Option<HashMap<Rc<str>, PathBuf>> = None;
        for table in tables {
            if table.name == "meta".into() {
                if let Some(nm) = table.get("name") {
                    if let Value::String(n) = nm {
                        name = Rc::from(n.as_str());
                    } else {
                        return Err(ManifestError::MissingName);
                    }
                }
                version = table.get("version").and_then(|ver| {
                    if let Value::String(v) = ver {
                        Some(Rc::from(v.as_str()))
                    } else {
                        None
                    }
                })
            } else if table.name == "build".into() {
                if let Some(cmp) = table.get("compiler") {
                    if let Value::String(c) = cmp {
                        compiler = Rc::from(c.as_str());
                    } else {
                        return Err(ManifestError::MissingCompiler);
                    }
                }
                if let Some(cflgs) = table.get("cflags")
                    && let Value::List(cfl) = cflgs
                {
                    cflags = cfl
                        .iter()
                        .filter_map(|val| {
                            if let Value::String(v) = val {
                                Some(Rc::from(v.as_str()))
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            } else if table.name == "dependencies".into() {
                let mut d = HashMap::new();
                for (dep, path) in table.iter() {
                    let key = dep.clone();
                    let path = if let Value::String(p) = path {
                        PathBuf::from(p)
                    } else {
                        return Err(ManifestError::Invalid);
                    };
                    d.insert(key, path);
                }
                deps = Some(d);
            }
        }

        Ok(Self {
            raw: src.into(),
            name,
            version,
            compiler,
            cflags,
            deps,
        })
    }
}

#[cfg(test)]
mod core_manifest_t {
    use std::rc::Rc;

    use crate::core::manifest::{EXAMPLE_MANIFEST, Manifest};

    #[test]
    fn parse_example_t() {
        let manifest_str = EXAMPLE_MANIFEST;
        let manifest = Manifest::parse_str(manifest_str).unwrap();
        assert_eq!(manifest.name.as_ref(), "main");
        assert_eq!(manifest.version, Some(Rc::<str>::from("0.1.0")));
        assert_eq!(manifest.compiler.as_ref(), "clang");
    }
}
