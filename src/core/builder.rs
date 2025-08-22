use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::core::{error::BuilderError, manifest::Manifest, utils::which};

pub struct Compiler {
    cmd: String,
    flags: Vec<String>,
}

impl Compiler {
    pub fn detect() -> Result<Self, BuilderError> {
        let cmd = if let Ok(cc) = std::env::var("CC")
            && let Some(_) = which(&cc)
        {
            cc
        } else if which("clang").is_some() {
            "clang".to_string()
        } else if which("gcc").is_some() {
            "gcc".to_string()
        } else {
            return Err(BuilderError::FailedToDetectCompiler);
        };

        Ok(Self {
            cmd,
            flags: Vec::new(),
        })
    }
    pub fn build(compiler: &str, flags: &[&str]) -> Result<Self, BuilderError> {
        let cmd = match compiler {
            "clang" => "clang".to_string(),
            "gcc" => "gcc".to_string(),
            _ => return Err(BuilderError::InvalidCompiler(compiler.to_string())),
        };

        let owned_flags = flags.iter().map(|flag| flag.trim().to_string()).collect();

        Ok(Self {
            cmd,
            flags: owned_flags,
        })
    }
    pub fn with_flags<S: AsRef<str>>(flags: &[S]) -> Result<Self, BuilderError> {
        let mut ret = Self::detect()?;
        ret.flags = flags.iter().map(|x| x.as_ref().to_string()).collect();
        Ok(ret)
    }
    pub fn add_flags<S: AsRef<str>>(&mut self, flags: &[S]) {
        for flag in flags {
            let str = flag.as_ref().to_string();
            self.flags.push(str);
        }
    }
    pub fn compile<P: AsRef<Path>>(&self, src: P, dst: P) -> Result<String, BuilderError> {
        let src = src.as_ref();
        let name = src
            .file_name()
            .map(|str| str.to_string_lossy())
            .ok_or(io::Error::last_os_error())?;

        if !src.exists() {
            return Err(BuilderError::CompileError(format!(
                "attempted to compile non-existent file '{name}'"
            )));
        }

        if !name.ends_with(".c") {
            return Err(BuilderError::CompileError(format!(
                "attempted to compile '{name}' which lacks the '.c' file extension."
            )));
        }

        let src_str = src.to_string_lossy();
        let dst_str = dst.as_ref().to_string_lossy();

        let command_str = format!(
            "{} -c -O2 -Iinclude {} -o {} {}",
            self.cmd,
            self.flags.join(" "),
            dst_str,
            src_str
        );

        let command = command_str.split_ascii_whitespace().collect::<Vec<_>>();
        let compile = Command::new(command[0]).args(&command[1..]).spawn()?;

        Ok(command_str)
    }
}

pub struct Builder<'a> {
    manifest: Manifest<'a>,
    compiler: Compiler,
    dir: PathBuf,
}

#[cfg(test)]
mod core_builder_t {
    use crate::core::builder::Compiler;

    #[ignore]
    #[test]
    fn compiler_t() {
        let compiler = Compiler::with_flags(&["-Wall", "-Wextra"]).unwrap();
        compiler
            .compile(
                "./tests/data/calc/src/main.c",
                "./tests/data/calc/build/main.o",
            )
            .unwrap();
    }
}
