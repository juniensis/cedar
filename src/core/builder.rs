use std::{
    fs, hash, io,
    ops::BitXor,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};

use crate::core::{error::BuilderError, manifest::Manifest, utils::which};

#[derive(Debug)]
pub struct Compiler {
    cmd: Rc<str>,
    flags: Vec<Rc<str>>,
}

impl Compiler {
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
    pub fn build(compiler: &str, flags: &[Rc<str>]) -> Result<Self, BuilderError> {
        let cmd = match compiler {
            "clang" => Rc::from(compiler),
            "gcc" => Rc::from(compiler),
            _ => return Err(BuilderError::InvalidCompiler(compiler.to_string())),
        };

        let owned_flags = flags.to_vec();

        Ok(Self {
            cmd,
            flags: owned_flags,
        })
    }
    pub fn with_flags<S: AsRef<str>>(flags: &[S]) -> Result<Self, BuilderError> {
        let mut ret = Self::detect()?;
        ret.flags = flags.iter().map(|x| Rc::from(x.as_ref())).collect();
        Ok(ret)
    }
    pub fn add_flags<S: AsRef<str>>(&mut self, flags: &[S]) {
        for flag in flags {
            let str = Rc::from(flag.as_ref());
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
            "{} -MMD -MF {}.d -c -O2 -Iinclude {} -o {} {}",
            self.cmd,
            dst_str,
            self.flags.join(" "),
            dst_str,
            src_str
        );

        let command = command_str.split_ascii_whitespace().collect::<Vec<_>>();
        let compile = Command::new(command[0]).args(&command[1..]).spawn()?;

        Ok(command_str)
    }
    pub fn link<P: AsRef<Path>>(&self, objects: &[P], dst: P) -> Result<String, BuilderError> {
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
                .map(|x| format!("{} ", x.as_ref().to_string_lossy()))
                .collect::<String>()
        );

        let command = command_str.split_ascii_whitespace().collect::<Vec<_>>();
        let link = Command::new(command[0])
            .args(&command[1..])
            .spawn()?
            .wait()?;

        Ok(command_str)
    }
}

#[derive(Debug)]
pub struct Builder {
    manifest: Manifest,
    compiler: Compiler,
    dir: PathBuf,
    build_dir: PathBuf,
    files: Vec<PathBuf>,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BuilderError> {
        let path = path.as_ref();
        let manifest_path = path.join("cedar.toml");
        if !manifest_path.exists() {
            return Err(BuilderError::NoManifest(format!("{path:?}")));
        }

        let manifest_str = fs::read_to_string(manifest_path)?;
        let manifest = Manifest::parse(&manifest_str)?;
        let compiler = manifest.compiler()?;
        let mut files = Vec::new();

        fn rec<P: AsRef<Path>>(out: &mut Vec<PathBuf>, path: P) -> io::Result<()> {
            let path = path.as_ref();
            for dir in path.read_dir()?.flatten() {
                let ft = dir.file_type()?;
                let pt = dir.path();
                if ft.is_dir() {
                    rec(out, pt)?;
                } else if ft.is_file() && pt.extension().is_some_and(|x| x == "c") {
                    out.push(pt);
                }
            }
            Ok(())
        }

        rec(&mut files, path)?;

        Ok(Self {
            manifest,
            compiler,
            dir: path.to_path_buf(),
            build_dir: path.join("build"),
            files,
        })
    }
    pub fn build(&self) -> Result<(), BuilderError> {
        if !self.build_dir.exists() {
            fs::create_dir_all(&self.build_dir)?;
        }

        let mut link = Vec::with_capacity(self.files.len());
        for file in &self.files {
            let hash = mangle_path(file);
            let outpath = self.dir.join("build").join(format!("{hash}.o"));
            self.compiler.compile(file, &outpath)?;
            link.push(outpath);
        }

        self.compiler
            .link(&link, self.build_dir.join(self.manifest.name.as_ref()))
            .unwrap();
        Ok(())
    }
}

const K: usize = 0x517cc1b727220a95;

fn mangle_path<P: AsRef<Path>>(path: P) -> String {
    let x = path
        .as_ref()
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0u64, |acc, byte| {
            acc.rotate_left(5)
                .bitxor(*byte as u64)
                .wrapping_mul(K as u64)
        });
    format!("{x:016x}")
}

#[cfg(test)]
mod core_builder_t {
    use crate::core::builder::{Builder, Compiler};

    const CALC_PATH: &str = "./tests/data/calc/";

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
    #[test]
    fn builder_t() {
        let calc_builder = Builder::new(CALC_PATH).unwrap();
        calc_builder.build().unwrap();
        println!("{calc_builder:?}");
    }
}
