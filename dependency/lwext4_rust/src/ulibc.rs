use alloc::alloc::{alloc, dealloc, Layout};
use alloc::string::String;
use core::cmp::min;
use core::ffi::{c_char, c_int, c_size_t, c_void};
use core::mem::size_of;
use core::ptr::copy_nonoverlapping;

#[cfg(feature = "print")]
#[linkage = "weak"]
#[no_mangle]
unsafe extern "C" fn printf(str: *const c_char, mut args: ...) -> c_int {
    use printf_compat::{format, output};
    let mut s = String::new();
    let bytes_written = format(str as _, args.as_va_list(), output::fmt_write(&mut s));
    trace!("[ext4] {}", s);
    bytes_written
}

#[cfg(not(feature = "print"))]
#[linkage = "weak"]
#[no_mangle]
unsafe extern "C" fn printf(_str: *const c_char, _args: ...) -> c_int {
    0
}

#[repr(transparent)]
struct MemTracker(usize);

/// Allocate size bytes memory and return the memory address.
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn malloc(len: c_size_t) -> *mut c_void {
    // Allocate `(actual length) + 8`. The lowest 8 Bytes are stored in the actual allocated space size.
    let layout = match Layout::from_size_align(size_of::<MemTracker>() + len, 8) {
        Ok(layout) => layout,
        Err(_) => {
            warn!("malloc failed: len = {}", len);
            return core::ptr::null_mut();
        }
    };
    unsafe {
        let ptr = alloc(layout).cast::<MemTracker>();
        assert!(!ptr.is_null(), "malloc failed");
        ptr.write(MemTracker(len));
        ptr.add(1).cast()
    }
}

/// Deallocate memory at ptr address
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        warn!("free a null pointer !");
        return;
    }

    let ptr = ptr.cast::<MemTracker>();
    unsafe {
        let ptr = ptr.sub(1);
        let len = ptr.read().0;
        let layout = match Layout::from_size_align(size_of::<MemTracker>() + len, 8) {
            Ok(layout) => layout,
            Err(_) => {
                warn!("free failed: invalid layout: len = {}", len);
                return;
            }
        };
        dealloc(ptr.cast(), layout)
    }
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn calloc(n: c_size_t, size: c_size_t) -> *mut c_void {
    let ptr = malloc(n * size);
    unsafe { ptr.write_bytes(0, n * size); }
    ptr
}

#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn realloc(old_ptr: *mut c_void, len: c_size_t) -> *mut c_void {
    if old_ptr.is_null() {
        warn!("realloc a a null mem pointer");
        return malloc(len);
    }

    let old_ptr = old_ptr.cast::<MemTracker>();
    let old_len = unsafe { old_ptr.sub(1).read().0 };
    let new_ptr = malloc(len);
    let copy_len = min(len, old_len);

    unsafe {
        copy_nonoverlapping(old_ptr.cast::<u8>(), new_ptr.cast::<u8>(), copy_len);
    }
    free(old_ptr.cast());
    new_ptr
}
