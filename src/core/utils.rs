use std::{
    ffi::c_void,
    io,
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

unsafe extern "C" {
    pub(crate) fn memchr(ptr: *const c_void, c: i32, len: usize) -> *const c_void;
}

/// Wrapper around memchr that bounds checks with a end pointer.
#[inline]
pub(crate) fn findbyte(ptr: *const u8, c: u8, end: *const u8) -> Result<*const u8, *const u8> {
    let rem = match (end as usize).checked_sub(ptr as usize) {
        Some(r) => r,
        None => return Err(ptr),
    };
    let ret = unsafe { memchr(ptr as *const c_void, c as i32, rem) } as *const u8;
    if ret.is_null() { Err(ptr) } else { Ok(ret) }
}

#[inline]
pub fn which(c: &str) -> Option<PathBuf> {
    let cmd = Command::new("which").arg(c).output().ok()?;

    let out = PathBuf::from(str::from_utf8(&cmd.stdout).ok()?.trim());
    if out.exists() { Some(out) } else { None }
}

pub fn walk_dir<P: AsRef<Path>>(root: P) -> io::Result<Vec<PathBuf>> {
    fn rec<P: AsRef<Path>>(out: &mut Vec<PathBuf>, cur: P) -> io::Result<()> {
        let path = cur.as_ref();
        for entry in path.read_dir()?.flatten() {
            let tp = entry.file_type()?;
            let pt = entry.path();
            if tp.is_dir() {
                rec(out, pt)?;
            } else if tp.is_file() {
                out.push(pt);
            }
        }
        Ok(())
    }
    let mut ret = Vec::new();
    rec(&mut ret, root)?;
    Ok(ret)
}

#[inline]
pub fn modified<P: AsRef<Path>>(path: P) -> Option<u64> {
    Some(
        path.as_ref()
            .metadata()
            .ok()?
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs(),
    )
}
