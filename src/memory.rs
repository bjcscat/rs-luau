use std::{
    alloc::{self, Layout},
    ffi::c_void,
    ptr::null_mut,
};

use crate::AssociatedData;

pub trait LuauAllocator {
    fn allocate(&self, size: usize) -> *mut c_void;
    fn reallocate(&self, ptr: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void;
    fn deallocate(&self, ptr: *mut c_void, old_size: usize);
}

#[cfg(any(
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "m68k",
    target_arch = "csky",
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "sparc",
    target_arch = "wasm32",
    target_arch = "hexagon",
    all(
        target_arch = "riscv32",
        not(any(target_os = "espidf", target_os = "zkvm"))
    ),
    all(target_arch = "xtensa", not(target_os = "espidf")),
))]
const SYS_MIN_ALIGN: usize = 8;

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "loongarch64",
    target_arch = "mips64",
    target_arch = "mips64r6",
    target_arch = "s390x",
    target_arch = "sparc64",
    target_arch = "riscv64",
    target_arch = "wasm64",
))]
const PLATFORM_ALIGNMENT: usize = 16;

#[cfg(any(
    all(target_arch = "riscv32", any(target_os = "espidf", target_os = "zkvm")),
    all(target_arch = "xtensa", target_os = "espidf"),
))]
const PLATFORM_ALIGNMENT: usize = 4;

pub struct DefaultLuauAllocator;

impl LuauAllocator for DefaultLuauAllocator {
    fn allocate(&self, size: usize) -> *mut c_void {
        let new_layout = match Layout::from_size_align(size, PLATFORM_ALIGNMENT) {
            Ok(layout) => layout,
            Err(_) => return null_mut(),
        };

        let new_ptr = unsafe { alloc::alloc(new_layout) as *mut c_void };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        new_ptr
    }

    fn reallocate(&self, ptr: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void {
        let old_layout = unsafe { Layout::from_size_align_unchecked(old_size, PLATFORM_ALIGNMENT) };

        let new_ptr = unsafe { alloc::realloc(ptr as *mut u8, old_layout, new_size) };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(old_layout);
        }

        new_ptr as _
    }

    fn deallocate(&self, ptr: *mut c_void, old_size: usize) {
        if ptr.is_null() {
            return;
        }

        unsafe {
            alloc::dealloc(
                ptr as _,
                Layout::from_size_align_unchecked(old_size, PLATFORM_ALIGNMENT),
            );
        }
    }
}

pub(crate) unsafe extern "C-unwind" fn luau_alloc_cb(
    ud: *mut c_void,
    ptr: *mut c_void,
    old_size: usize,
    new_size: usize,
) -> *mut c_void {
    let associated_data = ud.cast::<AssociatedData>().as_mut().unwrap();

    if new_size == 0 {
        if !ptr.is_null() {
            associated_data.allocator.deallocate(ptr, old_size);
        }

        return null_mut();
    }

    if new_size > isize::MAX as usize {
        return null_mut();
    }

    if ptr.is_null() {
        associated_data.allocator.allocate(new_size)
    } else {
        associated_data
            .allocator
            .reallocate(ptr, old_size, new_size)
    }
}
