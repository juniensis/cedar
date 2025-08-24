use std::{
    ffi::c_void,
    io,
    ops::BitXor,
    path::{Path, PathBuf},
    process::Command,
};

#[inline]
pub fn which(c: &str) -> Option<PathBuf> {
    let cmd = Command::new("which").arg(c).output().ok()?;

    let out = PathBuf::from(str::from_utf8(&cmd.stdout).ok()?.trim());
    if out.exists() { Some(out) } else { None }
}

#[inline]
pub(crate) fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

unsafe extern "C" {
    pub(crate) fn memchr(ptr: *const c_void, c: i32, len: usize) -> *const c_void;
}

/// Wrapper around memchr that bounds checks with a end pointer and returns
/// the pointer with the amount of characters jumped past.
#[inline]
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

pub fn walk_dir<P: AsRef<Path>>(root: P) -> std::io::Result<Vec<PathBuf>> {
    fn rec<P: AsRef<Path>>(out: &mut Vec<PathBuf>, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        for dir in path.read_dir()?.flatten() {
            let ft = dir.file_type()?;
            let pt = dir.path();
            if ft.is_dir() {
                rec(out, dir.path())?;
            } else if ft.is_file() {
                out.push(dir.path());
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    rec(&mut out, root)?;
    Ok(out)
}

const K: usize = 0x517cc1b727220a95;

#[inline]
pub fn fxhash(x: &[u8]) -> u64 {
    x.iter().fold(0u64, |acc, &byte| {
        acc.rotate_left(5)
            .bitxor(byte as u64)
            .wrapping_mul(K as u64)
    })
}

// Doesn't like to be inlined, -5%.
pub fn resolve_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let cmpiter = path
        .as_ref()
        .components()
        .filter(|cmp| cmp.as_os_str() != ".")
        .collect::<Vec<_>>();

    let mut out = PathBuf::from("./");
    let mut has_seen_non_dot = false;
    for cmp in cmpiter {
        if cmp.as_os_str() == ".." {
            if has_seen_non_dot {
                out.pop();
            } else {
                out.push(cmp);
            }
        } else {
            has_seen_non_dot = true;
            out.push(cmp);
        }
    }

    if out == PathBuf::from("./") {
        PathBuf::from(".")
    } else {
        out
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
