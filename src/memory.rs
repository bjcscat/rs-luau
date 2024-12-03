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

const PLATFORM_ALIGNMENT: usize = core::mem::align_of::<core::ffi::c_ulonglong>();

pub struct DefaultLuauAllocator {}

impl LuauAllocator for DefaultLuauAllocator {
    fn allocate(&self, size: usize) -> *mut c_void {
        let v =
            unsafe { alloc::alloc(Layout::from_size_align_unchecked(size, PLATFORM_ALIGNMENT)) };

        v as _
    }

    fn reallocate(&self, ptr: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void {
        let v = unsafe {
            let old_layout = Layout::from_size_align_unchecked(old_size, PLATFORM_ALIGNMENT);
            alloc::realloc(ptr as _, old_layout, new_size)
        };

        v as _
    }

    fn deallocate(&self, ptr: *mut c_void, old_size: usize) {
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
    if old_size == 0 {
        associated_data.allocator.allocate(new_size)
    } else if new_size == 0 {
        associated_data.allocator.deallocate(ptr, old_size);
        null_mut()
    } else {
        associated_data
            .allocator
            .reallocate(ptr, old_size, new_size)
    }
}
