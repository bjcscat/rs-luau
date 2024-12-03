mod ffi;
mod memory;
mod compile;
mod userdata;

use core::str;
use std::{any::Any, ffi::{c_char, c_int}, os::raw::c_void, ptr::null, slice};

use ffi::{
    luauconf::{LUAI_MAXCSTACK, LUA_MEMORY_CATEGORIES},
    prelude::*,
};
use memory::{luau_alloc_cb, DefaultLuauAllocator};
use userdata::{drop_userdata, dtor_rs_luau_userdata_callback, Userdata, UD_TAG};

pub use memory::LuauAllocator;

macro_rules! luau_stack_precondition {
    ($cond:expr) => {
        assert!(
            $cond,
            "Stack indicies should not exceed the top of the stack or extend below nor be a pseudo index other than an upvalue"
        )
    };
}

struct AssociatedData {
    allocator: Box<dyn LuauAllocator>,
    userdata: Option<Box<dyn Any>>,
}

/// Main struct implementing luau functionality
pub struct Luau {
    owned: bool,
    state: *mut _LuaState
}

impl Luau {
    unsafe fn new_state(allocator: impl LuauAllocator + 'static) -> *mut _LuaState {
        let associated_data = Box::new(AssociatedData {
            userdata: None,
            allocator: Box::new(allocator),
        });

        let state = lua_newstate(luau_alloc_cb, Box::into_raw(associated_data) as _);

        lua_setuserdatadtor(state, UD_TAG, Some(dtor_rs_luau_userdata_callback));

        (*lua_callbacks(state)).panic = Some(fatal_error_handler);

        state
    }

    pub fn new(allocator: impl LuauAllocator + 'static) -> Self {
        let state = unsafe { Self::new_state(allocator) };

        if state.is_null() {
            panic!("Initialization of Luau failed");
        }

        Self { owned: true, state }
    }

    /// Creates a Luau struct from a raw state pointer
    /// 
    /// # Safety
    /// The pointer must be a valid Luau state
    pub unsafe fn from_ptr(state: *mut _LuaState) -> Self {
        Self {
            owned: false,
            state
        }
    }

    pub fn set_data<T: Any>(&self, _: T) {
        // let boxed_data: Box<dyn Any> = Box::new(ud);
    }

    #[inline]
    pub fn to_ptr(&self) -> *mut _LuaState {
        self.state
    }

    #[inline]
    pub fn top(&self) -> c_int {
        unsafe { lua_gettop(self.state) }
    }

    /// Pops `n` values from the stack
    pub fn pop(&self, n: c_int) {
        // assert that the set position is not greater than the top
        luau_stack_precondition!(self.check_index(-n));

        // SAFETY: -n is validated by the precondition
        unsafe {lua_settop(self.state, -(n + 1))}
    }

    pub fn upvalue(&self, uv_idx: c_int) -> c_int {
        lua_upvalueindex(uv_idx)
    }

    pub fn set_memory_category(&self, cat: c_int) {
        assert!(
            cat < LUA_MEMORY_CATEGORIES,
            "Memory category index must not exceed {LUA_MEMORY_CATEGORIES}"
        );

        unsafe {
            lua_setmemcat(self.state, cat);
        }
    }

    pub fn check_index(&self, idx: c_int) -> bool {
        let top = self.top();

        if lua_ispseudo(idx) && (LUA_GLOBALSINDEX < idx) {
            return false; // do not permit pseudo indices except upvalues
        }

        let idx = if idx < 0 {
            // "subtract" the top (idx is negative)
            top.wrapping_add(idx)
        } else {
            idx
        };

        if idx < LUA_GLOBALSINDEX {
            // upvalue idx
            return true
        }

        idx >= 0 && // greater or equal to zero and
        idx <= top && // lesser than or equal to the top and
        idx < LUAI_MAXCSTACK // smaller than the maximum c stack
    }

    pub fn check_stack(&self, sz: c_int) -> bool {
        unsafe { lua_checkstack(self.state, sz) == 1 }
    }

    /// Will invoke Luau error code, if not called from a protected environment will cause a fatal error and panic
    // pub fn error(&self, error: LuauError) -> ! {
    //     match error {
    //         LuauError::AllocationError => unsafe {
    //             // how did we get here
    //             fatal_error_handler(self.state, LuaStatus::LUA_ERRMEM);
    //             std::process::abort();
    //         },
    //         LuauError::RuntimeError(contents) => unsafe {
    //             lua_pushlstring(self.state, contents.as_ptr() as _, contents.len());
    //             lua_error(self.state);
    //         },
    //     }
    // }

    /// Returns true if the value at `idx` is a bool, false otherwise
    pub fn is_boolean(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition
        unsafe {
            lua_isboolean(self.state, idx)
        }
    }

    /// Returns true if the value at `idx` is nil
    pub fn is_nil(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: stack size is validated by precondition
        unsafe {
            lua_isnil(self.state, idx)
        }
    }

    /// Pushes a nil value to the stack
    pub fn push_nil(&self) {
        luau_stack_precondition!(self.check_stack(1));
        
        // SAFETY: stack size is validated by precondition
        unsafe {
            lua_pushnil(self.state);
        }
    }

    /// Returns true if the value at `idx` is not nil or false, otherwise returns false
    pub fn to_boolean(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition
        unsafe {
            lua_toboolean(self.state, idx) == 1
        }
    }

    /// Pushes a boolean value to the Luau stack
    pub fn push_boolean(&self, value: bool) {
        luau_stack_precondition!(self.check_stack(1));

        // SAFETY: stack size is validated by the precondition
        unsafe {
            lua_pushboolean(self.state, value as i32);
        }
    }

    /// Returns true if the value at idx is a number, false otherwise
    pub fn is_number(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        unsafe { lua_isnumber(self.state, idx) == 1 }
    }

    /// Push a double into the Luau stack
    pub fn push_number(&self, n: f64) {
        // validate if the pushed index will not exceed the max C stack length
        luau_stack_precondition!(self.check_stack(1));

        // SAFETY: stack is appropriately sized, as checked by the precondition above
        unsafe {
            lua_pushnumber(self.state, n);
        }
    }

    /// Gets/converts a Lua value at `idx` to a number.
    ///
    /// Will convert a compatible string to a number
    pub fn to_number(&self, idx: c_int) -> Option<f64> {
        luau_stack_precondition!(self.check_index(idx));

        let mut is_number = 0;
        // SAFETY: idx is validated by the precondition and is therefore safe to access
        let number = unsafe { lua_tonumberx(self.state, idx, &raw mut is_number) };

        if is_number == 1 {
            Some(number)
        } else {
            None
        }
    }

    /// Returns true if the value at `idx` is a number, false otherwise
    pub fn is_string(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition
        unsafe { lua_isstring(self.state, idx) == 1 }
    }

    /// Pushes a string to the top of the Luau stack
    pub fn push_string(&self, str: impl AsRef<[u8]>) {
        luau_stack_precondition!(self.check_stack(1));

        let slice = str.as_ref();

        // SAFETY: the stack size is checked by the precondition
        unsafe {
            lua_pushlstring(self.state, slice.as_ptr() as _, slice.len());
        }
    }

    /// Gets or tries to coerce a Luau value at `idx` into a slice of u8s
    pub fn to_str_slice(&self, idx: c_int) -> Option<&[u8]> {
        luau_stack_precondition!(self.check_index(idx));

        // needs to have a lifetime to bind the result on a lifetime to prevent use after frees
        let mut len = 0;
        // SAFETY: idx is validated by the precondition
        let data = unsafe { lua_tolstring(self.state, idx, &mut len) };

        if !data.is_null() {
            // SAFETY: Luau can be trusted to return the correct len
            unsafe { Some(std::slice::from_raw_parts(data as _, len)) }
        } else {
            None
        }
    }

    /// Gets or tries to coerce a Luau value at `idx` into a str reference
    pub fn to_str(&self, idx: c_int) -> Option<Result<&str, str::Utf8Error>> {
        // preconditions are checked by to_string_slice
        self.to_str_slice(idx).map(|v| str::from_utf8(v))
    }

    /// Gets or converts a Luau value at `idx` into a string with a reasonable format, will invoke __tostring metamethods.
    pub fn convert_to_str_slice(&self, idx: c_int) -> &[u8] {
        luau_stack_precondition!(self.check_index(idx));

        unsafe {
            let mut len = 0;
            let data = luaL_tolstring(self.state, idx, &raw mut len);

            if data.is_null() {
                // shouldnt be possible
                panic!("Luau string conversion returned NULL ptr");
            } else {
                std::slice::from_raw_parts(data as _, len)
            }
        }
    }

    /// Returns true if the userdata at `idx` is a userdata and is of type T
    pub fn is_userdata<T: Any>(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition and the behavior of userdata is checked
        unsafe {
            let userdata_ptr: *mut Userdata<()> =
                lua_touserdatatagged(self.state, idx, UD_TAG) as _;

            !userdata_ptr.is_null() && (*userdata_ptr).is::<T>()
        }
    }

    /// Returns true if the userdata at `idx` is any type of userdata
    pub fn is_any_userdata<T: Any>(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition
        unsafe { lua_isuserdata(self.state, idx) == 1 }
    }

    /// Pushes a value T as a userdata to Luau
    pub fn push_userdata<T: Any>(&self, object: T) {
        luau_stack_precondition!(self.check_stack(1));

        // SAFETY: We allocate a DST as a userdata on a stack with the known proper size with our own tag.
        // if the userdat allo
        // if our type T has drop glue then we will set the dtor field which will be invoked
        // we then construct a struct which has ownership of T
        // we need the dtor field because the struct is opaque elsewhere
        unsafe {
            let userdata_ptr: *mut Userdata<T> =
                lua_newuserdatatagged(self.state, size_of::<Userdata<T>>(), UD_TAG).cast();

            let dtor = if std::mem::needs_drop::<T>() {
                let fn_item: unsafe fn(*mut Userdata<T>) = drop_userdata::<T>;

                Some(fn_item)
            } else {
                None
            };

            userdata_ptr.write(Userdata {
                id: object.type_id(),
                dtor,
                inner: object,
            });
        }
    }

    /// Gets a userdata value of type T, returning None if the value isn't a userdata or the userdata is not of type T
    pub fn get_userdata<T: Any>(&self, idx: c_int) -> Option<&mut T> {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: We validate that the userdata at the checked idx is of the proper type T and return a 'lua reference if so
        unsafe {
            let userdata_ptr: *mut Userdata<()> =
                lua_touserdatatagged(self.state, idx, UD_TAG) as _;

            if !userdata_ptr.is_null() && (*userdata_ptr).is::<T>() {
                Some(&mut (*(userdata_ptr as *mut Userdata<T>)).inner)
            } else {
                None
            }
        }
    }

    /// Retrives a userdata of type T without performing a type check to determine if the inner type is really T
    ///
    /// Will return None if the value at idx is not a userdata
    /// 
    /// # Safety
    /// You need to know beforehand that the userdata here is of the correct type or has such a layout that the type requested is valid
    pub unsafe fn get_userdata_unchecked<T: 'static>(
        &self,
        idx: c_int,
    ) -> Option<&mut T> {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: we don't do any checking other than validating idx
        unsafe {
            (lua_touserdatatagged(self.state, idx, UD_TAG) as *mut Userdata<T>)
                .as_mut()
                .map(|v| &mut v.inner)
        }
    }

    /// Returns true if the value at `idx` is a light userdata, it returns false otherwise.
    pub fn is_lightuserdata(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition
        unsafe {
            lua_islightuserdata(self.state, idx)
        }
    }

    /// Returns an option of a raw pointer. Will be Some if the value at `idx` is a lightuserdata, None otherwise.
    pub fn to_lightuserdata<T>(&self, idx: c_int) -> Option<*mut T> {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is checked by precondition
        unsafe {
            let ptr: *mut T = lua_tolightuserdata(self.state, idx).cast();

            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    /// Returns true if the Luau value at `idx` is a buffer, false otherwise
    pub fn is_buffer(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is checked by the precondition
        unsafe { lua_isbuffer(self.state, idx) }
    }

    /// Creates a luau buffer of a provided size and pushes it on the stack
    ///
    /// This will issue an error if the allocation cannot be performed
    pub fn push_buffer(&mut self, size: usize) -> &mut [u8] {
        luau_stack_precondition!(self.check_stack(1));

        unsafe {
            let ptr: *mut u8 = lua_newbuffer(self.state, size) as _;

            std::slice::from_raw_parts_mut(ptr, size)
        }
    }

    /// Pushes a slice to the Luau stack as a buffer
    pub fn push_buffer_from_slice(
        &mut self,
        slice: impl AsRef<[u8]>,
    ) -> &mut [u8] {
        // precondition is validated by push_buffer

        let slice = slice.as_ref();

        let buffer = self.push_buffer(slice.len());
        buffer.copy_from_slice(slice);

        buffer
    }

    /// Gets a Luau value at `idx` as a mutable slice of bytes
    pub fn to_buffer(&mut self, idx: c_int) -> Option<&mut [u8]> {
        luau_stack_precondition!(self.check_index(idx));

        let mut len = 0;
        // SAFETY: idx is validated by the precondition
        let data: *mut u8 = unsafe { lua_tobuffer(self.state, idx, &mut len) as _ };

        // will be null if the value is not a buffer
        if !data.is_null() {
            // SAFETY: Luau will report the right length
            unsafe { Some(slice::from_raw_parts_mut(data, len)) }
        } else {
            None
        }
    }

    /// Gets the pointer of a buffer value returning NULL if the value at `idx` is not a buffer
    pub fn to_buffer_ptr(&self, idx: c_int, len: &mut usize) -> *mut c_void {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by precondition
        unsafe { lua_tobuffer(self.state, idx, len) }
    }

    /// Returns true if the value at `idx` is a function, false otherwise
    pub fn is_function(&self, idx: c_int) -> bool {
        luau_stack_precondition!(self.check_index(idx));

        // SAFETY: idx is validated by the precondition
        unsafe { lua_isfunction(self.state, idx) }
    }

    /// Pushes a raw rust function to the stack which receives a pointer to the luau state and returns the number of result values
    ///
    /// Can receive a number of upvalues specified by the `num_upvalues` argument which are accessed through ffi's upvalueindex
    /// 
    /// # Safety
    /// You will need to uphold all safety invariants with respect to the Luau VM in the user supplied `func`
    pub unsafe fn push_raw_function(
        &self,
        func: unsafe extern "C-unwind" fn(*mut _LuaState) -> c_int,
        debug_name: Option<&str>,
        num_upvals: c_int,
    ) {
        luau_stack_precondition!(self.check_stack(1));

        assert!(
            self.top() >= num_upvals,
            "The number of upvalues for a raw function must not exceed the stack length"
        );
        
        // SAFETY: upvalue count and stack size are validated as a precondition and assert
        unsafe {
            lua_pushcclosure(self.state, func, if let Some(name) = debug_name {
                name.as_ptr() as *const c_char
            } else {
                null()
            }, num_upvals);
        }
    }

    /// Pushes a Rust function into Luau
    ///
    /// This function wraps a Rust function to allow closures to capture values, to avoid this minor overhead you can use `push_function_raw`
    pub fn push_function<F: Fn(Luau) -> i32>(&self, func: F, debug_name: Option<&str>, num_upvals: c_int) {
        assert!(
            self.top() >= num_upvals,
            "The number of upvalues for a raw function must not exceed the stack length"
        );    

        luau_stack_precondition!(self.check_stack(2));
        
        let func_box = Box::new(func);

        unsafe extern "C-unwind" fn invoke_fn<T: Fn(Luau) -> i32>(state: *mut _LuaState) -> c_int {
            let func = lua_tolightuserdata(state, lua_upvalueindex(1)).cast::<T>();

            (*func)(Luau::from_ptr(state))
        }

        unsafe {
            lua_pushlightuserdata(self.state, Box::into_raw(func_box) as _);

            self.push_raw_function(invoke_fn::<F>, debug_name, 1 + num_upvals);
        }
    }

    /// Calls a Luau function returning the status of the Luau state when it returns
    pub fn call(&self, nargs: c_int, nresults: c_int) -> LuaStatus {
        assert!(
            self.is_function(-1),
            "The value at top of stack must be a function"
        );

        assert!(
            self.top() >= nargs,
            "Argument count may not exceed the total stack size"
        );

        luau_stack_precondition!(self.check_stack(nresults));

        unsafe { lua_pcall(self.state, nargs, nresults, 0) }
    }
}

// TODO: do this
unsafe extern "C-unwind" fn fatal_runtime_error_handler(state: *mut _LuaState) -> c_int {
    let luau = unsafe { Luau::from_ptr(state) };

    panic!("Uncaught runtime error - \"{}\"", String::from_utf8_lossy(luau.convert_to_str_slice(-1)));
}

/// Final resting place for Luau code, we don't return from this.
unsafe extern "C-unwind" fn fatal_error_handler(state: *mut _LuaState, status: LuaStatus) {
    match status {
        // Unhandled runtime error
        LuaStatus::LUA_ERRRUN => fatal_runtime_error_handler(state),
        // memory allocation error, just die
        LuaStatus::LUA_ERRMEM => std::process::abort(),
        // some error handling mechanism errored
        LuaStatus::LUA_ERRERR => panic!("Error originating from error handling mechanism"),
        // shouldnt be reachable
        _ => unreachable!(),
    };

    panic!("Fatal error in Luau execution");
}

impl Default for Luau {
    fn default() -> Self {
        Self::new(DefaultLuauAllocator {})
    }
}

impl Drop for Luau {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }

        unsafe {
            lua_close(self.state);
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod binding_tests {
    use std::{ffi::{c_int, c_void}, hint::black_box, rc::Rc};

    use crate::{Luau, LuauAllocator, _LuaState, lua_error, lua_tonumber, lua_upvalueindex};

    #[test]
    #[should_panic]
    fn stack_checking_no_value() {
        let luau = Luau::default();

        luau.is_number(1);
    }

    #[test]
    #[should_panic]
    fn stack_checking_neg_no_value() {
        let luau = Luau::default();

        luau.is_number(-1);
    }

    #[test]
    fn stack_checking_has_value() {
        let luau = Luau::default();

        luau.push_number(0.0);

        luau.is_number(-1);
        luau.is_number(1);
        luau.is_number(0); // not the value but is the nil value
    }

    #[test]
    #[should_panic]
    fn unhandled_error() {
        let luau = Luau::default();

        luau.push_string("hello error!");

        unsafe {
            lua_error(luau.to_ptr());
        }
    }

    #[test]
    fn pop() {
        let luau = Luau::default();

        luau.push_number(0.0);

        assert_eq!(luau.top(), 1);

        luau.pop(1);

        assert_eq!(luau.top(), 0);

        luau.push_number(0.0);
        luau.push_number(0.0);

        assert_eq!(luau.top(), 2);

        luau.pop(2);

        assert_eq!(luau.top(), 0);
    }

    #[test]
    fn function_upvalue_test() {
        let luau = Luau::default();
        
        luau.push_number(1.0);
        luau.push_number(2.0);
        luau.push_number(3.0);

        luau.push_function(|luau| {
            assert_eq!(luau.to_number(luau.upvalue(1)), Some(1.0));
            assert_eq!(luau.to_number(luau.upvalue(2)), Some(2.0));
            assert_eq!(luau.to_number(luau.upvalue(3)), Some(3.0));

            0
        }, Some("test"), 3);

        luau.call(0, 0);
    }

    #[test]
    fn raw_function_upvalue_test() {
        let luau = Luau::default();
        
        luau.push_number(1.0);
        luau.push_number(2.0);
        luau.push_number(3.0);

        unsafe extern "C-unwind" fn test_extern_fn(state: *mut _LuaState) -> c_int {
            assert_eq!(lua_tonumber(state, lua_upvalueindex(1)), 1.0);
            assert_eq!(lua_tonumber(state, lua_upvalueindex(2)), 2.0);
            assert_eq!(lua_tonumber(state, lua_upvalueindex(3)), 3.0);

            0
        }

unsafe {
        luau.push_raw_function( test_extern_fn, Some("test"), 3);
    }
        luau.call(0, 0);
    }

    #[test]
    fn luau_panic_unwind() {
        struct PanicAllocator {}

        impl LuauAllocator for PanicAllocator {
            fn allocate(&self, _: usize) -> *mut std::ffi::c_void {
                panic!()
            }

            fn reallocate(&self, _: *mut c_void, _: usize, _: usize) -> *mut std::ffi::c_void {
                panic!()
            }

            fn deallocate(&self, _: *mut c_void, _: usize) {
                panic!()
            }
        }

        assert!(std::panic::catch_unwind(|| {
            black_box(Luau::new(PanicAllocator {}));
        })
        .is_err());
    }

    // #[test]
    // fn try_safety() {
    // let luau = Luau::default();

    // let mut did_error = false;

    // luau.lua_try_catch(
    //     |state, _| state.error(LuauError::RuntimeError("error!")),
    //     |_, did_error| {
    //         *did_error = true;
    //     },
    //     &mut did_error,
    // );

    // assert!(did_error, "Expected error callback to be invoked.")

    // todo!();
    // }

    #[test]
    fn userdata_values() {
        let luau = Luau::default();

        luau.push_userdata(());

        let mut vec = Vec::with_capacity(128);
        for i in 0..128 {
            vec.push(i);
        }

        luau.push_userdata(vec);

        #[repr(transparent)]
        struct DropCheck(Rc<()>);

        let drop_rc = Rc::new(());
        let yes_drop = DropCheck(drop_rc.clone());

        luau.push_userdata(yes_drop);

        assert!(luau.get_userdata(-3).copied() == Some(()));
        assert!(luau
            .get_userdata::<Vec<i32>>(-2)
            .is_some_and(|v| v.is_sorted())); // is larger data preserved correctly
        assert!(luau.get_userdata::<DropCheck>(-1).is_some());

        drop(luau);

        // assert!(, "Expected userdata to be dropped with luau state");
    }

    #[test]
    fn string_values() {
        let luau = Luau::default();

        const TEST_CONST: &[u8] = &[0xCA, 0xFE, 0xBA, 0xBE];
        const INVALID_SEQUENCE: &[u8] = &[0xC3, 0x28];

        luau.push_string("Hello, world!");
        luau.push_string(TEST_CONST);
        luau.push_number(12345.0f64);
        luau.push_string(INVALID_SEQUENCE);

        assert_eq!(luau.to_str_slice(-4), Some(b"Hello, world!" as _));
        assert_eq!(luau.to_str(-4), Some(Ok("Hello, world!")));
        assert_eq!(luau.to_str_slice(1), Some(b"Hello, world!" as _));
        assert_eq!(luau.to_str(1), Some(Ok("Hello, world!")));

        assert_eq!(luau.to_str_slice(-3), Some(TEST_CONST));
        assert_eq!(luau.to_str_slice(2), Some(TEST_CONST));

        assert_eq!(luau.to_str_slice(-2), Some(b"12345" as _));
        assert_eq!(luau.to_str(-2), Some(Ok("12345")));
        assert_eq!(luau.to_str_slice(3), Some(b"12345" as _));
        assert_eq!(luau.to_str(3), Some(Ok("12345")));

        assert_eq!(luau.to_str_slice(-1), Some(INVALID_SEQUENCE));
        assert!(luau.to_str(-1).is_some_and(|r| r.is_err()));
        assert_eq!(luau.to_str_slice(4), Some(INVALID_SEQUENCE));
        assert!(luau.to_str(4).is_some_and(|r| r.is_err()));
    }

    #[test]
    fn numeric_values() {

        let luau = Luau::default();

        luau.push_number(f64::NAN);
        luau.push_number(f64::INFINITY);
        luau.push_number(f64::EPSILON);
        luau.push_string("12345");

        // nan is not equal to itself, because that makes sense
        assert_eq!(luau.to_number(-4).map(f64::to_bits), Some(f64::NAN.to_bits()));
        assert_eq!(luau.to_number(1).map(f64::to_bits), Some(f64::NAN.to_bits()));

        assert_eq!(luau.to_number(-3), Some(f64::INFINITY));
        assert_eq!(luau.to_number(2), Some(f64::INFINITY));

        assert_eq!(luau.to_number(-2), Some(f64::EPSILON));
        assert_eq!(luau.to_number(3), Some(f64::EPSILON));

        assert_eq!(luau.to_number(-1), Some(12345.0f64));
        assert_eq!(luau.to_number(4), Some(12345.0f64));
    }
}
