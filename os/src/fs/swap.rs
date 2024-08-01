use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

use crate::{arch::BLOCK_SZ, config::PAGE_SIZE, drivers::BLOCK_DEVICE};

// use super::directory_tree::FILE_SYSTEM;
use lazy_static::*;

lazy_static! {
    pub static ref SWAP_DEVICE: Mutex<Swap> = Mutex::new(Swap::new(16));
}

#[derive(Debug)]
pub struct SwapTracker(pub usize);

impl Drop for SwapTracker {
    fn drop(&mut self) {
        SWAP_DEVICE.lock().discard(self.0);
    }
}

pub struct Swap {
    bitmap: Vec<u64>,
    block_ids: Vec<u64>,
}
const BLK_PER_PG: usize = PAGE_SIZE / BLOCK_SZ;
const SWAP_SIZE: usize = 1024 * 1024;
impl Swap {
    /// size: the number of megabytes in swap
    pub fn new(_size: usize) -> Self {
        // TODO: impl this in ext4
        Self {
            bitmap: Vec::new(),
            block_ids: Vec::new(),
        }
        // let bit = size * (SWAP_SIZE / PAGE_SIZE); // 1MiB = 4KiB*256
        // let vec_len = bit / usize::MAX.count_ones() as usize;
        // let mut bitmap = Vec::<u64>::with_capacity(vec_len);
        // bitmap.resize(bitmap.capacity(), 0);
        // let blocks = size * (SWAP_SIZE / BLOCK_SZ); // 1MiB = 512B * 2048
        // Self {
        //     bitmap,
        //     block_ids: FILE_SYSTEM.alloc_blocks(blocks),
        // }
    }
    fn read_page(block_ids: &[u64], buf: &mut [u8]) {
        assert!(block_ids[0] + BLK_PER_PG as u64 - 1 == block_ids[BLK_PER_PG - 1]);
        BLOCK_DEVICE.read_block(block_ids[0], buf);
    }
    fn write_page(block_ids: &[u64], buf: &[u8]) {
        assert!(block_ids[0] + (BLK_PER_PG as u64 - 1) == block_ids[BLK_PER_PG - 1]);
        BLOCK_DEVICE.write_block(block_ids[0], buf);
    }
    fn set_bit(&mut self, pos: u64) {
        self.bitmap[pos as usize / 64] |= 1 << (pos % 64);
    }
    fn clear_bit(&mut self, pos: usize) {
        self.bitmap[pos / 64] &= !(1 << (pos % 64));
    }
    fn alloc_page(&self) -> Option<u64> {
        for (i, bit) in self.bitmap.iter().enumerate() {
            if !*bit == 0 {
                continue;
            }
            return Some(i as u64 * 64 + (!*bit).trailing_zeros() as u64);
        }
        None
    }
    fn get_block_ids(&self, swap_id: u64) -> &[u64] {
        &self.block_ids[swap_id as usize * BLK_PER_PG + 0..swap_id as usize * BLK_PER_PG+ BLK_PER_PG]
    }
    pub fn read(&mut self, swap_id: u64, buf: &mut [u8]) {
        Self::read_page(self.get_block_ids(swap_id), buf);
    }
    pub fn write(&mut self, buf: &[u8]) -> Arc<SwapTracker> {
        let swap_id:u64 = self.alloc_page().unwrap();
        Self::write_page(self.get_block_ids(swap_id), buf);
        self.set_bit(swap_id);
        Arc::new(SwapTracker(swap_id as usize))
    }
    #[inline(always)]
    pub fn discard(&mut self, swap_id: usize) {
        self.clear_bit(swap_id);
    }
}
