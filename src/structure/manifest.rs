use serde::{Deserialize, Serialize};
use toml::{self};

use super::project::ProjectError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub meta: Meta,
    pub build: Build,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Meta {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Build {
    pub compiler: String,
    pub cflags: Vec<String>,
}

impl Manifest {
    #[inline]
    pub fn parse(manifest: &str) -> Result<Self, ProjectError> {
        let parsed = match toml::from_str(manifest) {
            Ok(manifest) => manifest,
            Err(_) => return Err(ProjectError::InvalidManifest),
        };

        Ok(parsed)
    }
    #[inline]
    pub fn as_string(&self) -> Result<String, ProjectError> {
        let serialized = match toml::ser::to_string(self) {
            Ok(manifest) => manifest,
            Err(_) => return Err(ProjectError::InvalidManifest),
        };

        Ok(serialized)
    }
    pub fn new() -> Self {
        Manifest {
            meta: Meta {
                name: String::new(),
                version: Some(String::from("0.1.0")),
                description: None,
            },
            build: Build {
                compiler: String::from("GCC"),
                cflags: vec![String::from("-Wall"), String::from("-Wextra")],
            },
        }
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Manifest;

    #[test]
    fn test_deserialize() {
        let file = fs::read_to_string("./data/cedar.toml").unwrap();

        let parsed = Manifest::parse(&file).unwrap();

        println!("{:?}", parsed);
    }
}
