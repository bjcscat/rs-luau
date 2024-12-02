use std::os::raw::{c_char, c_int};
use std::ptr;

#[repr(C)]
pub struct LuaCompileOptions {
    /// Determines the degree of optimizations the compiler will do
    ///
    /// 0. No optimizations
    /// 1. Optimizations which will not impact debuggability
    /// 2. All optimizations in level 1 plus optimizations that harm debuggability such as inlining
    pub optimization_level: c_int,
    
    /// Determiens the degree to which debugging information will be included
    ///
    /// 0. No debug information
    /// 1. Line info & function names only; sufficient for backtraces
    /// 2. Full debug info with local & upvalue names; necessary for debugger
    pub debug_level: c_int,

    /// Type information is used to guide native code generation decisions
    ///
    /// Information includes testable types for function arguments, locals, upvalues and some temporaries
    ///
    /// 0. generate for native modules
    /// 1. generate for all modules
    pub type_info_level: c_int,

    /// Determines the degree to which converage information should be included into the bytecode
    pub coverage_level: c_int,

    /// Global library to construct vectors; disabled by default
    pub vector_lib: *const c_char,

    /// Global builtin to construct vectors; disabled by default
    pub vector_ctor: *const c_char,

    /// Vector typename for type tables; disabled by default
    pub vector_type: *const c_char,

    /// `NULL`-terminated array of globals that are mutable; disables import optimizations for fields accessed through these
    pub mutable_globals: *const *const c_char,

    /// `NULL`-terminated array of userdata types which will be included in the type information
    pub userdata_types: *const *const c_char,
}

impl Default for LuaCompileOptions {
    fn default() -> Self {
        Self {
            optimization_level: 1,
            debug_level: 1,
            type_info_level: 0,
            coverage_level: 0,
            vector_lib: ptr::null(),
            vector_ctor: ptr::null(),
            vector_type: ptr::null(),
            mutable_globals: ptr::null(),
            userdata_types: ptr::null(),
        }
    }
}

extern "C-unwind" {
    /// Compiles Luau source into Luau bytecode.
    ///
    /// This code will not hold references to the source code after calling allowing it to be freed.
    ///
    /// This code will set the value at `outsize` if it is not NULL
    ///
    /// This will return NULL on an allocation error and an error encoded in the resultant bytecode string on a compilation error
    pub fn luau_compile(
        source: *const c_char,
        size: usize,
        options: *mut LuaCompileOptions,
        outsize: *mut usize,
    ) -> *mut c_char;
}
