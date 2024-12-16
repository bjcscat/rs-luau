
use std::ffi::c_int;

use super::luau::_LuaState;

pub const LUA_COLIBNAME: &str = "coroutine";
pub const LUA_TABLIBNAME: &str = "table";
pub const LUA_OSLIBNAME: &str = "os";
pub const LUA_STRLIBNAME: &str = "string";
pub const LUA_BITLIBNAME: &str = "bit32";
pub const LUA_BUFFERLIBNAME: &str = "buffer";
pub const LUA_UTF8LIBNAME: &str = "utf8";
pub const LUA_MATHLIBNAME: &str = "math";
pub const LUA_DBLIBNAME: &str = "debug";

extern "C-unwind" {
    pub fn luaopen_base(L: *mut _LuaState) -> c_int;
    pub fn luaopen_coroutine(L: *mut _LuaState) -> c_int;
    pub fn luaopen_table(L: *mut _LuaState) -> c_int;
    pub fn luaopen_os(L: *mut _LuaState) -> c_int;
    pub fn luaopen_string(L: *mut _LuaState) -> c_int;
    pub fn luaopen_bit32(L: *mut _LuaState) -> c_int;
    pub fn luaopen_buffer(L: *mut _LuaState) -> c_int;
    pub fn luaopen_utf8(L: *mut _LuaState) -> c_int;
    pub fn luaopen_math(L: *mut _LuaState) -> c_int;
    pub fn luaopen_debug(L: *mut _LuaState) -> c_int;

    // open all builtin libraries
    pub fn luaL_openlibs(L: *mut _LuaState);
}