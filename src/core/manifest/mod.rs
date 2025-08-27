use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};

use crate::core::{
    build::compiler::Compiler,
    error::{BuilderError, ManifestError},
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
        Self::parse(src.as_bytes())
    }
    pub fn parse(src: &[u8]) -> Result<Self, ManifestError> {
        let tables = toml_parse(src);
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
            raw: Rc::from(str::from_utf8(src).unwrap()),
            name,
            version,
            compiler,
            cflags,
            deps,
        })
    }
    pub fn new(name: &str) -> Result<Self, ManifestError> {
        let compiler = Compiler::detect().unwrap();
        let comp_str = compiler.as_str();
        let manifest_str = manifest_with_name(name, comp_str);
        Self::parse_str(&manifest_str)
    }
    pub fn write<P: AsRef<Path>>(&self, out: P) -> io::Result<()> {
        fs::write(out, self.raw.as_bytes())
    }
    pub fn compiler(&self) -> Result<Compiler, BuilderError> {
        Compiler::new(self.compiler.clone(), &self.cflags)
    }
}

fn manifest_with_name(name: &str, compiler: &str) -> String {
    format!(
        "[meta]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[build]\ncompiler = \"{compiler}\"\ncflags = [\"-Wall\", \"-Wextra\"]\n"
    )
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
