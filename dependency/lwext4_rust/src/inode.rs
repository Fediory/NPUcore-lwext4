use core::ops::Deref;
use crate::bindings::*;

pub struct Ext4InodeRef(pub(crate) ext4_inode_ref);

unsafe impl Send for Ext4InodeRef {}

impl Deref for Ext4InodeRef {
    type Target = ext4_inode;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.inode }
    }
}

impl Ext4InodeRef {
    pub fn update(&mut self, f: impl FnOnce(&mut ext4_inode)) {
        unsafe { f(&mut *self.0.inode); }
        self.0.dirty = true;
    }
}
