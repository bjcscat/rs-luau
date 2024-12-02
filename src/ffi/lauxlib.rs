use std::ffi::{c_char, c_float, c_int, c_uint, c_void};

use super::luau::{CFunction, LuaInteger, LuaNumber, LuauType, LuaUnsigned, _LuaState};

#[repr(C)]
pub struct LuaLibRegister {
    name: *const c_char,
    func: CFunction,
}

extern "C-unwind" {
    /// Opens a library.
    ///
    /// When called with libname equal to NULL, it simply registers all functions in the list l (see luaL_Reg) into the table on the top of the stack.
    ///
    /// When called with a non-null libname, luaL_register creates a new table t, sets it as the value of the global variable libname, sets it as the value of _LOADED\[libname] where loaded is a registry table, and registers on it all functions in the list l. If there is a table in _LOADED\[libname] or in variable libname, reuses this table instead of creating a new one.
    ///
    /// In any case the function leaves the table on the top of the stack.
    pub fn luaL_register(state: *mut _LuaState, libname: *const c_char, l: *mut LuaLibRegister);

    /// Pushes onto the stack the field `e` from the metatable of the object at index `obj`.
    /// 
    /// If the object does not have a metatable, or if the metatable does not have this field, returns false and pushes nothing.
    pub fn luaL_getmetafield(state: *mut _LuaState, obj: c_int, e: *const c_char) -> c_int;

    /// Calls a metamethod.
    ///
    /// If the object at index `obj` has a metatable and this metatable has a field `e`, this function calls this field passing the object as its only argument.
    /// In this case this function returns true and pushes onto the stack the value returned by the call. 
    /// 
    /// If there is no metatable or no metamethod, this function returns false (without pushing any value on the stack).
    pub fn luaL_callmeta(state: *mut _LuaState, obj: c_int, e: *const c_char) -> c_int;

    /// Emits a preformatted type error where `tname` is the expected type.
    ///
    /// If the value at stack index narg doesnt exist it will emit a "missing" type error
    ///
    /// If the value DOES exist then it will emit a "expected X got Y" error
    pub fn luaL_typeerrorL(state: *mut _LuaState, narg: c_int, tname: *const c_char) -> !;

    /// Emits a preformatted argument error where `narg` is formatted as a number.
    ///
    /// If a function name is associated with the currently executing function then it will be included in the formatted message.
    pub fn luaL_argerrorL(state: *mut _LuaState, narg: c_int, extramsg: *const c_char) -> !;

    /// Checks whether the function argument `narg` is a string and returns this string; if `l` is not NULL fills its referent with the string's length.
    ///
    /// This function uses `lua_tolstring` to get its result, so all conversions and caveats of that function apply here.
    pub fn luaL_checklstring(state: *mut _LuaState, narg: c_int, l: *mut usize) -> *const c_char;

    /// If the function argument `narg` is a string, returns this string. If this argument is absent or is nil, returns `d`. Otherwise, raises an error.
    ///
    /// If `l` is not NULL, fills its referent with the result's length. If the result is NULL (only possible when returning `d` and `d` == `NULL`), its length is considered zero.
    ///
    /// This function uses `lua_tolstring` to get its result, so all conversions and caveats of that function apply here.
    pub fn luaL_optlstring(
        state: *mut _LuaState,
        narg: c_int,
        def: *const c_char,
        l: *mut usize,
    ) -> *const c_char;

    /// Checks whether the function argument arg is a number and returns this number.
    pub fn luaL_checknumber(state: *mut _LuaState, numArg: c_int) -> LuaNumber;

    /// If the function argument `narg` is a number, returns this number as a lua_Number.
    /// If this argument is absent or is nil, returns `def`. Otherwise, raises an error.
    pub fn luaL_optnumber(state: *mut _LuaState, narg: c_int, def: LuaNumber) -> LuaNumber;

    /// Checks whether the function argument arg is a boolean and returns it.
    pub fn luaL_checkboolean(state: *mut _LuaState, narg: c_int) -> c_int;

    /// Checks whether the function argument `narg` is a boolean and returns it.
    /// If this argument is absent or is nil, returns `def`. Otherwise, raises an error.
    pub fn luaL_optboolean(state: *mut _LuaState, narg: c_int, def: c_int) -> c_int;

    /// Checks whether the function argument `narg` is an integer (or can be converted to an integer) and returns this integer.
    pub fn luaL_checkinteger(state: *mut _LuaState, numArg: c_int) -> LuaInteger;

    /// If the function argument `narg` is an integer (or it is convertible to an integer), returns this integer. 
    /// If this argument is absent or is `nil`, returns `def`. Otherwise, raises an error.
    pub fn luaL_optinteger(state: *mut _LuaState, nArg: c_int, def: LuaInteger) -> LuaInteger;

    /// Checks whether the function argument `narg` is an unsigned integer (or can be converted to an unsigned integer) and returns this unsigned integer.
    pub fn luaL_checkunsigned(state: *mut _LuaState, numArg: c_int) -> LuaUnsigned;

    /// If the function argument `narg` is an unsigned integer (or it is convertible to an unsigned integer), returns this unsigned integer.
    /// If this argument is absent or is nil, returns `def`. Otherwise, raises an error.
    pub fn luaL_optunsigned(state: *mut _LuaState, numArg: c_int, def: LuaUnsigned) -> LuaUnsigned;

    /// Checks whether the function argument `narg` is a vector and returns this vector.
    pub fn luaL_checkvector(state: *mut _LuaState, narg: c_int) -> *const c_float;

    /// If the function argument `narg` is a vector, returns this vector. If this argument is absent or is nil, returns `def`. Otherwise, raises an error.
    pub fn luaL_optvector(
        state: *mut _LuaState,
        narg: c_int,
        def: *const c_float,
    ) -> *const c_float;

    /// Grows the stack size to `top + sz` elements, raising an error if the stack cannot grow to that size.
    /// 
    /// `msg` is an additional text to go into the error message (or `NULL` for no additional text).
    pub fn luaL_checkstack(state: *mut _LuaState, sz: c_int, msg: *const c_char);

    /// Checks whether the function argument `narg` is of LuaType t
    pub fn luaL_checktype(state: *mut _LuaState, narg: c_int, t: LuauType);

    /// Checks for an argument of any type (including nil), errors if the argument is omitted entirely
    pub fn luaL_checkany(state: *mut _LuaState, narg: c_int);

    /// If the registry already has the key `tname`, returns 0. 
    /// Otherwise, creates a new table to be used as a metatable for userdata, adds to this new table the pair __name = tname, adds to the registry the pair [tname] = new table, and returns 1.
    /// 
    /// In both cases, the function pushes onto the stack the final value associated with `tname` in the registry.
    pub fn luaL_newmetatable(state: *mut _LuaState, tname: *const c_char) -> c_int;

    /// Checks whether the function argument `narg` is a userdata of the type tname (see luaL_newmetatable) and returns the userdata's address.
    pub fn luaL_checkudata(state: *mut _LuaState, ud: c_int, tname: *const c_char) -> *mut c_void;

    /// Checks whether the function argument `narg` is a buffer of and returns this buffer and sets the value at the len pointer to the correct value.
    pub fn luaL_checkbuffer(state: *mut _LuaState, narg: c_int, len: *mut usize) -> *mut c_void;

    /// Pushes onto the stack a string identifying the current position of the control at level `lvl` in the call stack. Typically this string has the following format:
    /// 
    /// `chunkname:currentline:`
    /// 
    /// Level 0 is the running function, level 1 is the function that called the running function, etc.
    /// 
    /// This function is used to build a prefix for error messages.
    pub fn luaL_where(state: *mut _LuaState, lvl: c_int);

    /// Errors a Luau state with a snprintf formatted string
    pub fn luaL_errorL(state: *mut _LuaState, fmt: *const c_char, ...) -> !;

    /// Checks whether the function argument `narg` is a string and searches for this string in the array lst (which must be `NULL`-terminated). 
    /// 
    /// Returns the index in the array where the string was found. Raises an error if the argument is not a string or if the string cannot be found.
    /// 
    /// If `def` is not `NULL`, the function uses `def` as a default value when there is no argument arg or when this argument is nil.
    pub fn luaL_checkoption(
        state: *mut _LuaState,
        narg: c_int,
        def: *const c_char,
        lst: *const *const c_char,
    ) -> c_int;

    /// Converts any Luau value at the given index to a C string in a reasonable format.
    /// 
    /// The resulting string is pushed onto the stack and also returned by the function. If `len` is not `NULL`, the function also sets `*len` with the string length.
    /// 
    /// If the value has a metatable with a `__tostring` field, then luaL_tolstring calls the corresponding metamethod with the value as argument, and uses the result of the call as its result.
    pub fn luaL_tolstring(state: *mut _LuaState, idx: c_int, len: *mut usize) -> *const c_char;

    /// Creates a Luau state with a default (system) allocator function
    pub fn luaL_newstate() -> *mut _LuaState;

    /// Pushes the value at string index `fname` in a table at `idx` to the top of the stack.
    /// 
    /// `szhint` should be a length of the the `fname`.
    /// 
    /// Will return `NULL` if no match is found and fname if a match is found.
    pub fn luaL_findtable(
        state: *mut _LuaState,
        idx: c_int,
        fname: *const c_char,
        szhint: c_int,
    ) -> *const c_char;

    /// Returns the typename of a value at `idx`
    pub fn luaL_typename(state: *mut _LuaState, idx: c_int) -> *const c_char;
}
