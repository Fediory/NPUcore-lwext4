#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(asm_experimental_arch)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(int_roundings)]
#![feature(string_remove_matches)]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(trait_upcasting)]

pub use arch::config;
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod arch;
mod drivers;
mod fs;
mod lang_items;
mod mm;
mod syscall;
mod task;
mod timer;
mod net;
mod utils;

use crate::arch::{bootstrap_init, machine_init};
#[cfg(feature = "block_mem")]
use crate::config::DISK_IMAGE_BASE;
#[cfg(feature = "la64")]
core::arch::global_asm!(include_str!("arch/la64/entry.asm"));
#[cfg(feature = "block_mem")]
core::arch::global_asm!(include_str!("load_img.S"));
#[cfg(not(feature = "block_mem"))]
core::arch::global_asm!(include_str!("preload_app.S"));

fn mem_clear() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    #[cfg(feature = "zero_init")]
    unsafe {
        core::slice::from_raw_parts_mut(
            sbss as usize as *mut u8,
            crate::config::MEMORY_END - sbss as usize,
        )
        .fill(0);
    }
    #[cfg(not(feature = "zero_init"))]
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[cfg(feature = "block_mem")]
fn move_to_high_address() {
    extern "C" {
        fn simg();
        fn eimg();
    }
    unsafe {
        let img = core::slice::from_raw_parts(
            simg as usize as *mut u8,
            eimg as usize - simg as usize
        );
        // 从DISK_IMAGE_BASE到MEMORY_END
        let mem_disk = core::slice::from_raw_parts_mut(
            DISK_IMAGE_BASE as *mut u8,
            0x800_0000
        );
        mem_disk.fill(0);
        mem_disk[..img.len()].copy_from_slice(img);
    }
}

#[no_mangle]
pub fn rust_main() -> ! {
    bootstrap_init();
    mem_clear();
    println!("NPUcore-IMPACT ENTER!!!");
    #[cfg(feature = "block_mem")]
    move_to_high_address();
    console::log_init();
    println!("[kernel] Console initialized.");
    mm::init();
    // note that remap_test is currently NOT supported by LA64, for the whole kernel space is RW!
    //mm::remap_test();

    machine_init();
    println!("[kernel] Hello, world!");
    
    //machine independent initialization
    // use crate::drivers::block::block_device_test;
    // block_device_test();
    fs::directory_tree::init_fs();
    net::config::init();
    #[cfg(not(feature = "block_mem"))]
    fs::flush_preload();
    task::add_initproc();

    // note that in run_tasks(), there is yet *another* pre_start_init(),
    // which is used to turn on interrupts in some archs like LoongArch.
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

#[cfg(test)]
fn test_runner(_tests: &[&dyn Fn()]) {}
