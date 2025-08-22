use std::{collections::HashMap, fmt::Debug, path::Path};

use crate::core::error::ManifestError;

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

pub struct Manifest<'a> {
    raw: &'a str,
    name: &'a str,
    version: Option<&'a str>,
    compiler: &'a str,
    cflags: Vec<&'a str>,
    deps: Option<HashMap<&'a str, &'a Path>>,
}

impl<'a> Manifest<'a> {
    pub fn parse(src: &'a str) -> Result<Self, ManifestError> {
        let mut name = None;
        let mut version = None;
        let mut compiler = None;
        let mut cflags = Vec::new();
        //let mut deps = None;

        for line in src.lines() {
            let split = line.split("=").collect::<Vec<_>>();
            if split.len() == 2 {
                match split[0].trim() {
                    "name" => name = Some(split[1].trim().trim_matches('"').trim()),
                    "version" => version = Some(split[1].trim().trim_matches('"').trim()),
                    "compiler" => compiler = Some(split[1].trim().trim_matches('"').trim()),
                    "cflags" => {
                        for flag in split[1].split(",").map(|flag| {
                            flag.trim()
                                .trim_matches('[')
                                .trim_matches(']')
                                .trim_matches('"')
                                .trim()
                        }) {
                            cflags.push(flag);
                        }
                    }
                    _ => {}
                }
            }
        }

        if let (Some(n), v, Some(c)) = (name, version, compiler) {
            Ok(Self {
                raw: src,
                name: n,
                version: v,
                compiler: c,
                cflags,
                deps: None,
            })
        } else if name.is_none() {
            Err(ManifestError::MissingName)
        } else if compiler.is_none() {
            Err(ManifestError::MissingCompiler)
        } else {
            Err(ManifestError::Invalid)
        }
    }
}

impl<'a> Debug for Manifest<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.raw)?;
        writeln!(
            f,
            "name: {}, {}compiler: {}, flags: {:?}",
            self.name,
            if let Some(v) = self.version {
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
    use crate::core::manifest::{EXAMPLE_MANIFEST, Manifest};

    #[test]
    fn parse_t() {
        let mani = Manifest::parse(EXAMPLE_MANIFEST).unwrap();
        assert_eq!(mani.name, "main");
        assert_eq!(mani.version, Some("0.1.0"));
        assert_eq!(mani.compiler, "clang");
        assert_eq!(mani.cflags, vec!["-Wall", "-Wextra"]);
    }
}
