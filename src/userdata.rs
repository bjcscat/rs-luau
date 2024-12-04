use std::{
    any::{Any, TypeId},
    cell::Cell,
    error::Error,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    os::raw::c_void,
    ptr::drop_in_place,
};

use crate::ffi::{luauconf::LUA_UTAG_LIMIT, prelude::*};

pub(crate) const UD_TAG: Tag = Tag(LUA_UTAG_LIMIT - 1);

#[derive(Debug)]
#[repr(C)]
pub(crate) struct Userdata<T: Any + ?Sized> {
    pub(crate) id: TypeId, // typeid of T
    pub(crate) count_cell: Cell<isize>,
    pub(crate) dtor: Option<unsafe fn(*mut Userdata<T>)>,
    pub(crate) inner: T,
}

impl<T: Any + ?Sized> Userdata<T> {
    pub(crate) fn is<V: Any>(&self) -> bool {
        self.id == TypeId::of::<V>()
    }
}

#[derive(Debug)]
pub enum UserdataBorrowError {
    AlreadyMutable,
    AlreadyImmutable,
}

impl Display for UserdataBorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserdataBorrowError::AlreadyImmutable => {
                write!(f, "Cannot mutably borrow userdata, is already borrowed.")
            }
            UserdataBorrowError::AlreadyMutable => {
                write!(f, "Cannot borrow userdata, is already mutably borrowed.")
            }
        }
    }
}

impl Error for UserdataBorrowError {}

pub struct UserdataRef<T: Any>(*mut Userdata<T>);

impl<T: Any> UserdataRef<T> {
    pub(crate) unsafe fn try_from_ptr(
        value: *mut Userdata<T>,
    ) -> Result<UserdataRef<T>, UserdataBorrowError> {
        let v = (*value).count_cell.get();
        match v {
            -1 => Err(UserdataBorrowError::AlreadyMutable),
            _ => {
                (*value).count_cell.set(v + 1);

                Ok(Self(value))
            }
        }
    }
}

impl<T: Any> Deref for UserdataRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: cant be initialized with a null pointer
        unsafe { &self.0.as_ref().unwrap_unchecked().inner }
    }
}

impl<T: Any> Drop for UserdataRef<T> {
    fn drop(&mut self) {
        unsafe {
            let v = (*self.0).count_cell.get();
            (*self.0).count_cell.set(v - 1)
        }
    }
}

pub struct UserdataRefMut<T: Any>(*mut Userdata<T>);

impl<T: Any> UserdataRefMut<T> {
    pub(crate) unsafe fn try_from_ptr(
        value: *mut Userdata<T>,
    ) -> Result<Self, UserdataBorrowError> {
        let v = (*value).count_cell.get();
        match v {
            0 => {
                (*value).count_cell.set(-1);

                Ok(Self(value))
            }
            -1 => Err(UserdataBorrowError::AlreadyMutable),
            _ => Err(UserdataBorrowError::AlreadyImmutable),
        }
    }
}

impl<T: Any> Deref for UserdataRefMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: cant be initialized with a null pointer
        unsafe { &(*self.0).inner }
    }
}

impl<T: Any> DerefMut for UserdataRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: cant be initialized with a null pointer
        unsafe { &mut (*self.0).inner }
    }
}

impl<T: Any> Drop for UserdataRefMut<T> {
    fn drop(&mut self) {
        unsafe { (*self.0).count_cell.set(0) }
    }
}

// This function is some cursed stuff.
// it derefs the pointer as *mut Userdata<()> to get a zero sized field so it can read the dtor
pub(crate) unsafe extern "C-unwind" fn dtor_rs_luau_userdata_callback(
    _: *mut _LuaState,
    v: *mut c_void,
) {
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
    fn exec_test() {}
}
