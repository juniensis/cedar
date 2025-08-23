use std::{
    ffi::c_void,
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

unsafe extern "C" {
    pub(crate) fn memchr(ptr: *const c_void, c: i32, len: usize) -> *const c_void;
}

/// Wrapper around memchr that bounds checks with a end pointer and returns
/// the pointer with the amount of characters jumped past.
pub(crate) fn findbyte(ptr: *const u8, c: u8, end: *const u8) -> Option<(*const u8, usize)> {
    let rem = (end as usize).checked_sub(ptr as usize)?;
    let ret = unsafe { memchr(ptr as *const c_void, c as i32, rem) } as *const u8;
    if ret.is_null() {
        None
    } else {
        let jumped = (ret as usize) - (ptr as usize);
        Some((ret, jumped))
    }
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
