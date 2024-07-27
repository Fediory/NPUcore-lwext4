mod block_dev;
mod mem_blk;
mod sata_blk;
pub use block_dev::BlockDevice;
#[cfg(feature = "block_mem")]
pub type BlockDeviceImpl = mem_blk::MemBlockWrapper;
#[cfg(feature = "block_sata")]
pub type BlockDeviceImpl = sata_blk::SataBlock;

use crate::arch::BLOCK_SZ;
use alloc::sync::Arc;
use lazy_static::*;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; BLOCK_SZ];
    let mut read_buffer = [0u8; BLOCK_SZ];
    for i in 0..BLOCK_SZ {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as u64, &write_buffer);
        block_device.read_block(i as u64, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}
