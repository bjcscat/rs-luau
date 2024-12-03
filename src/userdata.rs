use std::{any::{Any, TypeId}, fmt::Debug, os::raw::c_void, ptr::drop_in_place};

use crate::ffi::{luauconf::LUA_UTAG_LIMIT, prelude::*};

pub(crate) const UD_TAG: Tag = Tag(LUA_UTAG_LIMIT - 1);

#[derive(Debug)]
#[repr(C)]
pub(crate) struct Userdata<T: Any + ?Sized> {
    pub(crate) id: TypeId, // typeid of T
    pub(crate) dtor: Option<unsafe fn(*mut Userdata<T>)>,
    pub(crate) inner: T
}

impl<T: Any + ?Sized> Userdata<T> {
    pub(crate) fn is<V: Any>(&self) -> bool {
        self.id == TypeId::of::<V>()
    }
}

pub(crate) struct UserdataRef<T: Any>(*mut Userdata<T>);

// This function is some cursed stuff.
// it derefs the pointer as *mut Userdata<()> to get a zero sized field so it can read the dtor
pub(crate) unsafe extern "C-unwind" fn dtor_rs_luau_userdata_callback(_: *mut _LuaState, v: *mut c_void) {
    let mut_self = &mut *(v as *mut Userdata<()>);

    mut_self.dtor.inspect(|func| {
        func(v as _);
    });
}

// needs to invoke drop_in_place for T
pub(crate) unsafe fn drop_userdata<T: Any + ?Sized>(ud: *mut Userdata<T>) {
    drop_in_place(&raw mut (*ud).inner);
}

#[cfg(test)]
mod tests {
    #[test]
    fn exec_test() {
    }
}