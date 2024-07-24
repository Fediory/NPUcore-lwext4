use alloc::ffi::CString;
use alloc::string::{String, ToString};
use crate::bindings::*;

pub struct Ext4DirIter(ext4_dir);

pub struct Ext4DirEntry<'a> {
    pub inode: u32,
    pub name: &'a str,
    pub type_: u8,
}

impl Ext4DirIter {
    pub(crate) fn new(f: ext4_file) -> Self {
        Self(ext4_dir {
            f,
            de: unsafe { core::mem::zeroed() },
            next_off: 0,
        })
    }
}

impl Ext4DirIter {
    pub fn next(&mut self) -> Option<Ext4DirEntry> {
        unsafe {
            if ext4_dir_entry_next(&mut self.0).is_null() {
                return None;
            }
        };
        let name_buf = &self.0.de.name[..self.0.de.name_length as usize];
        Some(Ext4DirEntry {
            inode: self.0.de.inode,
            name: core::str::from_utf8(&name_buf).unwrap(),
            type_: self.0.de.inode_type,
        })
    }
}

pub fn lwext4_symlink(src: &str, target: &str) -> Result<(), i32> {
    let src_path = CString::new(src).unwrap();
    let target_path = CString::new(target).unwrap();
    match unsafe { ext4_fsymlink(target_path.as_ptr(), src_path.as_ptr()) } {
        0 => Ok(()),
        e => {
            error!("ext4_fsymlink {} -> {} failed: {}", src, target, e);
            Err(e)
        }
    }
}

pub fn lwext4_rmdir(path: &str) -> Result<(), i32> {
    let c_path = CString::new(path).unwrap();
    match unsafe { ext4_dir_rm(c_path.as_ptr()) } {
        0 => Ok(()),
        e => {
            error!("ext4_dir_rm {} failed: {}", path, e);
            Err(e)
        }
    }
}

pub fn lwext4_rmfile(path: &str) -> Result<(), i32> {
    let c_path = CString::new(path).unwrap();
    match unsafe { ext4_fremove(c_path.as_ptr()) } {
        0 => Ok(()),
        e => {
            error!("ext4_dir_rm {} failed: {}", path, e);
            Err(e)
        }
    }
}

pub fn lwext4_movedir(old_path: &str, new_path: &str) -> Result<(), i32> {
    let c_old_path = CString::new(old_path).unwrap();
    let c_new_path = CString::new(new_path).unwrap();
    match unsafe { ext4_dir_mv(c_old_path.as_ptr(), c_new_path.as_ptr()) } {
        0 => Ok(()),
        e => {
            error!("ext4_dir_mv {} to {} failed: {}", old_path, new_path, e);
            Err(e)
        }
    }
}

pub fn lwext4_movefile(old_path: &str, new_path: &str) -> Result<(), i32> {
    let c_old_path = CString::new(old_path).unwrap();
    let c_new_path = CString::new(new_path).unwrap();
    match unsafe { ext4_frename(c_old_path.as_ptr(), c_new_path.as_ptr()) } {
        0 => Ok(()),
        e => {
            error!("ext4_dir_mv {} to {} failed: {}", old_path, new_path, e);
            Err(e)
        }
    }
}

pub fn lwext4_readlink(path: &str) -> Result<String, i32> {
    let c_path = CString::new(path).unwrap();
    let mut buf = [0u8; 260];
    let mut rcnt = 0;
    unsafe {
        match ext4_readlink(c_path.as_ptr(), buf.as_mut_ptr() as _, 260, &mut rcnt) {
            0 => Ok(String::from_utf8_lossy(&buf[..rcnt as usize]).to_string()),
            e => {
                error!("ext4_readlink {} failed: {}", path, e);
                Err(e)
            }
        }
    }
}
