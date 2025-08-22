use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn which(c: &str) -> Option<PathBuf> {
    let cmd = Command::new("which").arg(c).output().ok()?;

    let out = PathBuf::from(str::from_utf8(&cmd.stdout).ok()?.trim());
    if out.exists() { Some(out) } else { None }
}

pub(crate) fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

#[cfg(test)]
mod core_sys_t {
    use std::path::PathBuf;

    use crate::core::utils::which;

    #[test]
    fn which_t() {
        let exp_1 = PathBuf::from("/usr/bin/clang");
        let exp_2 = PathBuf::from("/usr/bin/gcc");

        assert_eq!(Some(exp_1), which("clang"));
        assert_eq!(Some(exp_2), which("gcc"));
        assert_eq!(None, which("nonexistent_program"));
    }
}
