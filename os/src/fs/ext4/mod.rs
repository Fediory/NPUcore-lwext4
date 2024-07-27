use ext4_view::{Ext4, Ext4Error, PathBuf};
use alloc::boxed::Box;
use alloc::vec::Vec;

fn load_test_disk1() -> Ext4 {
    const DATA: &[u8] = include_bytes!("../../../../easy-fs-fuse/ext4.bin");
    Ext4::load(Box::new(DATA.to_vec())).unwrap()
}

pub fn init_ext4fs() {
    let fs = load_test_disk1();
    let dir = fs
        .read_dir("/bin")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

}