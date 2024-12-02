use std::os::raw::c_int;

use super::luau::_LuaState;

extern "C-unwind" {
    /// Returns 0 if codegen is not supported on the current platform.
    /// 
    /// Returns 1 if codegen is supported
    pub fn luau_codegen_supported() -> c_int;
    /// Creates a codegen environment for the luau state, must be called before `luau_codegen_compile`
    pub fn luau_codegen_create(state: *mut _LuaState);
    /// Compiles a luau function with native code generation
    pub fn luau_codegen_compile(state: *mut _LuaState, idx: c_int);
}