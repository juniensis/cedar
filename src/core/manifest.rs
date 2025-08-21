use std::{collections::HashMap, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::builder::Compiler;

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub meta: Meta,
    pub build: Build,
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Build {
    pub compiler: String,
    pub cflags: Option<Vec<String>>,
}

impl Manifest {
    pub fn parse(manifest: &str) -> Self {
        toml::from_str(manifest).unwrap()
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let read = fs::read(path)?;
        Ok(toml::from_slice(&read).unwrap())
    }
    pub fn new(name: &str) -> Self {
        Manifest {
            meta: Meta {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: None,
            },
            build: Build {
                compiler: Compiler::detect().as_string(),
                cflags: Some(vec!["-Wall".to_string(), "-Wextra".to_string()]),
            },
            dependencies: None,
        }
    }
}

#[cfg(test)]
mod core_manifest_t {
    use crate::core::manifest::Manifest;

    #[test]
    fn parse_t() {
        let test = include_str!("../../tests/project/cedar.toml");
        let manifest = Manifest::parse(test);
        println!("{manifest:?}");
    }
}
