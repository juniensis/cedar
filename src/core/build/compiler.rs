use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    rc::Rc,
};

use crate::core::{error::BuilderError, manifest::Manifest, utils::which};

#[derive(Debug)]
pub struct Compiler {
    cmd: Rc<str>,
    flags: Vec<Rc<str>>,
}

impl Compiler {
    /// Checks if the 'CC' environment variable is set, use that as the
    /// compiler, otherwise, check if 'clang' exists, if so use it, then
    /// fallback to 'gcc'.
    pub fn detect() -> Result<Self, BuilderError> {
        let cmd = if let Ok(cc) = std::env::var("CC")
            && let Some(_) = which(&cc)
        {
            Rc::from(cc.as_str())
        } else if which("clang").is_some() {
            Rc::from("clang")
        } else if which("gcc").is_some() {
            Rc::from("clang")
        } else {
            return Err(BuilderError::FailedToDetectCompiler);
        };

        Ok(Self {
            cmd,
            flags: Vec::new(),
        })
    }
    /// Creates a new instance from strings.
    pub fn new<S: AsRef<str>>(compiler: S, flags: &[S]) -> Result<Self, BuilderError> {
        let cmd = match compiler.as_ref() {
            "clang" => Rc::from(compiler.as_ref()),
            "gcc" => Rc::from(compiler.as_ref()),
            _ => return Err(BuilderError::InvalidCompiler(compiler.as_ref().to_string())),
        };

        let owned_flags = flags
            .iter()
            .map(|flag| Rc::<str>::from(flag.as_ref()))
            .collect::<Vec<_>>();

        Ok(Self {
            cmd,
            flags: owned_flags,
        })
    }
    /// Autodetect compiler, and create an instance withh the given flags.
    pub fn with_flags<S: AsRef<str>>(flags: &[S]) -> Result<Self, BuilderError> {
        let mut ret = Self::detect()?;
        ret.flags = flags.iter().map(|x| Rc::from(x.as_ref())).collect();
        Ok(ret)
    }
    /// Add flags to an existing compiler.
    pub fn add_flags<S: AsRef<str>>(&mut self, flags: &[S]) {
        for flag in flags {
            let str = Rc::from(flag.as_ref());
            self.flags.push(str);
        }
    }
    /// Compile a file. Runs the following command (if clang is used with
    /// -Wall and -Wextra):
    ///
    /// 'clang -MMD -MF dst.d -c -O2 -Iinclude -Wall -Wextra -o dst.o src'
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
            "{} -MMD -MF {}.d -c -O2 -Iinclude {} -o {}.o {}",
            self.cmd,
            dst_str,
            self.flags.join(" "),
            dst_str,
            src_str
        );

        let command = command_str.split_ascii_whitespace().collect::<Vec<_>>();
        let compile = Command::new(command[0])
            .args(&command[1..])
            .spawn()?
            .wait()?;

        if compile.success() {
            Ok(command_str)
        } else {
            Err(BuilderError::CompileError(format!(
                "Compiler failed to run, {command_str}"
            )))
        }
    }
    /// Links the given .o files into a binary specified by 'dst'.
    pub fn link<P: AsRef<Path>>(&self, objects: &[P], dst: &P) -> Result<String, BuilderError> {
        let dst = dst.as_ref();
        if let Some(par) = dst.parent() {
            if !par.exists() {
                fs::create_dir_all(par)?;
            }
        } else {
            return Err(BuilderError::CompileError(format!(
                "Invalid destination path: {dst:?}"
            )));
        }

        let command_str = format!(
            "{} -o {} {} -lm",
            self.cmd,
            dst.to_string_lossy(),
            objects
                .iter()
                .map(|x| format!("{}.o ", x.as_ref().to_string_lossy()))
                .collect::<String>()
        );

        let command = command_str.split_ascii_whitespace().collect::<Vec<_>>();
        let link = Command::new(command[0])
            .args(&command[1..])
            .spawn()?
            .wait()?;

        if link.success() {
            Ok(command_str)
        } else {
            Err(BuilderError::CompileError(format!(
                "Linker failed to run, {command_str}"
            )))
        }
    }
}
