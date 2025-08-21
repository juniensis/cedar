use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::core::{manifest::Manifest, sys::which};

pub enum Compiler {
    Clang(PathBuf),
    Gcc(PathBuf),
    Env(PathBuf),
    Unknown,
}

impl Compiler {
    pub fn detect() -> Compiler {
        if let Ok(cc) = std::env::var("CC")
            && let Some(path) = which(&cc)
        {
            return Compiler::Env(path);
        }
        if let Some(clang) = which("clang") {
            return Compiler::Clang(clang);
        }
        if let Some(gcc) = which("gcc") {
            return Compiler::Gcc(gcc);
        }
        Compiler::Unknown
    }
    pub fn as_string(&self) -> String {
        match self {
            Self::Clang(_) => "clang".to_string(),
            Self::Gcc(_) => "gcc".to_string(),
            Self::Env(p) => p.file_name().unwrap().to_string_lossy().to_string(),
            Self::Unknown => "Unknown".to_string(),
        }
    }
}

pub struct Builder {
    compiler: Compiler,
    cflags: Vec<String>,
    bin: String,
    directory: PathBuf,
}

impl Builder {
    pub fn build_all<P: AsRef<Path>>(&self, paths: &[P]) -> io::Result<()> {
        let obj_dir = self.directory.join("obj/");
        if !obj_dir.exists() {
            fs::create_dir(&obj_dir)?;
        }

        let obj_str = obj_dir.into_os_string();
        let mut to_link = Vec::new();
        for path in paths {
            let path_str = path.as_ref().to_string_lossy();
            let path_name = path.as_ref().file_stem().unwrap().to_string_lossy();
            let obj_path = format!("{}/{path_name}.o", obj_str.display());
            to_link.push(obj_path.clone());
            let args = [
                "-c",
                "-O2",
                "-Iinclude",
                "-o",
                obj_path.as_str(),
                path_str.as_ref(),
            ];

            let mut op = Command::new(self.compiler.as_string().as_str())
                .args(&self.cflags)
                .args(args)
                .spawn()?;

            op.wait()?;
        }

        let bin_dir = self
            .directory
            .join(&self.bin)
            .into_os_string()
            .into_string()
            .unwrap();

        let mut link = Command::new(self.compiler.as_string().as_str())
            .args(["-o", &bin_dir])
            .args(&to_link)
            .arg("-lm")
            .spawn()?;

        link.wait()?;

        Ok(())
    }
}

#[cfg(test)]
mod core_builder_t {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::core::builder::{Builder, Compiler};

    #[test]
    fn build_all_t() {
        let builder = Builder {
            compiler: Compiler::detect(),
            cflags: vec!["-Wall".to_string(), "-Wextra".to_string()],
            bin: "main".to_string(),
            directory: PathBuf::from("./tests/project/build"),
        };

        let mut to_build = Vec::new();
        fn rec<P: AsRef<Path>>(out: &mut Vec<PathBuf>, path: P) {
            for dir in fs::read_dir(path).unwrap().flatten() {
                if dir.file_type().unwrap().is_dir() {
                    rec(out, dir.path());
                } else if dir.file_type().unwrap().is_file()
                    && dir.file_name().into_string().unwrap().ends_with(".c")
                {
                    out.push(dir.path());
                }
            }
        }

        rec(&mut to_build, "./tests/project/");

        builder.build_all(&to_build).unwrap();
        println!("{to_build:?}");
    }
}
