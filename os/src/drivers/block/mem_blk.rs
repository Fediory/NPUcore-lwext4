use super::BlockDevice;
use crate::{arch::BLOCK_SZ, config::DISK_IMAGE_BASE};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use spin::Mutex;
struct MemBlock(usize);

impl MemBlock {
    const BLOCK_SIZE: usize = BLOCK_SZ;
    pub fn block_ref(&self, block_id: usize, len: usize) -> &[u8] {
        unsafe { from_raw_parts((self.0 + block_id * Self::BLOCK_SIZE) as *const u8, len) }
    }
    pub fn block_refmut(&self, block_id: usize, len: usize) -> &mut [u8] {
        unsafe { from_raw_parts_mut((self.0 + block_id * Self::BLOCK_SIZE) as *mut u8, len) }
    }
}

pub struct MemBlockWrapper(Mutex<MemBlock>);

#[allow(unused)]
impl MemBlockWrapper {
    const BASE_ADDR: usize = DISK_IMAGE_BASE;
    pub fn new() -> Self {
        Self(Mutex::new(MemBlock(MemBlockWrapper::BASE_ADDR)))
    }
}
use log::info;
impl BlockDevice for MemBlockWrapper {
    fn read_block(&self, block_id: u64, buf: &mut [u8]) {
        info!("[mem read_block] len : {}", buf.len());
        let blk = self.0.lock();
        buf.copy_from_slice(blk.block_ref(block_id as usize, buf.len()));
    }
    fn write_block(&self, block_id: u64, buf: &[u8]) {
        info!("[mem write_block] len : {}", buf.len());
        let blk = self.0.lock();
        blk.block_refmut(block_id as usize, buf.len()).copy_from_slice(buf);
    }
}

impl lwext4_rs::BlockDeviceInterface for MemBlockWrapper {
    fn open(&mut self) -> lwext4_rs::Result<lwext4_rs::BlockDeviceConfig> {
        Ok(lwext4_rs::BlockDeviceConfig {
            block_size: BLOCK_SZ as u32,
            block_count: 999,
            part_size: BLOCK_SZ as u64 * 2,
            part_offset: 0,
        })
    }

    fn read_block(
        &mut self,
        buf: &mut [u8],
        block_id: u64,
        block_count: u32,
    ) -> lwext4_rs::Result<usize> {
        let blk = self.0.lock();
        buf.copy_from_slice(blk.block_ref(block_id as usize, BLOCK_SZ * block_count as usize));
        Ok(0)
    }
    fn write_block(
        &mut self,
        buf: &[u8],
        block_id: u64,
        block_count: u32,
    ) -> lwext4_rs::Result<usize> {
        let blk = self.0.lock();
        blk.block_refmut(block_id as usize, BLOCK_SZ * block_count as usize)
            .copy_from_slice(buf);
        Ok(0)
    }

    fn close(&mut self) -> lwext4_rs::Result<()> {
        Ok(())
    }

    fn lock(&mut self) -> lwext4_rs::Result<()> {
        Ok(())
    }

    fn unlock(&mut self) -> lwext4_rs::Result<()> {
        Ok(())
    }
}