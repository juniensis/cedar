use serde::{Deserialize, Serialize};
use toml::{self};

use crate::error::ManifestError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub meta: Meta,
    pub build: Build,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Meta {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Build {
    pub compiler: String,
    pub cflags: Vec<String>,
}

impl Manifest {
    #[inline]
    pub fn parse(manifest: &str) -> Result<Self, ManifestError> {
        let parsed = match toml::from_str(manifest) {
            Ok(manifest) => manifest,
            Err(e) => return Err(ManifestError::InvalidManifest(e)),
        };

        Ok(parsed)
    }
    #[inline]
    pub fn as_string(&self) -> Result<String, ManifestError> {
        let serialized = match toml::ser::to_string(self) {
            Ok(manifest) => manifest,
            Err(e) => return Err(ManifestError::SerializeError(e)),
        };

        Ok(serialized)
    }
    pub fn new() -> Self {
        Manifest {
            meta: Meta {
                name: String::new(),
                version: String::from("0.1.0"),
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
