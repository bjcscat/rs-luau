use std::ffi::{c_char, c_int};
use std::ffi::{c_double, c_float, c_void};
use std::ptr::{self, null};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum LuauBytecodeType {
    LBC_TYPE_NIL = 0,
    LBC_TYPE_BOOLEAN,
    LBC_TYPE_NUMBER,
    LBC_TYPE_STRING,
    LBC_TYPE_TABLE,
    LBC_TYPE_FUNCTION,
    LBC_TYPE_THREAD,
    LBC_TYPE_USERDATA,
    LBC_TYPE_VECTOR,
    LBC_TYPE_BUFFER,

    LBC_TYPE_ANY = 15,

    LBC_TYPE_TAGGED_USERDATA_BASE = 64,
    LBC_TYPE_TAGGED_USERDATA_END = 64 + 32,

    LBC_TYPE_OPTIONAL_BIT = 1 << 7,

    LBC_TYPE_INVALID = 256,
}

pub type LuauCompilerConstant = *mut c_void;

/// return a type identifier for a global library member
pub type LuauLibraryMemberTypeCallback =
    unsafe extern "C-unwind" fn(library: *const c_char, member: *const c_char) -> LuauBytecodeType;

// setup a value of a constant for a global library member
// use luau_set_compile_constant_*** set of functions for values
pub type LuauLibraryMemberConstantCallback = unsafe extern "C-unwind" fn(
    library: *const c_char,
    member: *const c_char,
    constant: LuauCompilerConstant,
);

#[repr(C)]
#[allow(non_snake_case)]
pub struct LuauCompileOptions {
    /// Determines the degree of optimizations the compiler will do
    ///
    /// 0. No optimizations
    /// 1. Optimizations which will not impact debuggability
    /// 2. All optimizations in level 1 plus optimizations that harm debuggability such as inlining
    pub optimizationLevel: c_int,

    /// Determiens the degree to which debugging information will be included
    ///
    /// 0. No debug information
    /// 1. Line info & function names only; sufficient for backtraces
    /// 2. Full debug info with local & upvalue names; necessary for debugger
    pub debugLevel: c_int,

    /// Type information is used to guide native code generation decisions
    ///
    /// Information includes testable types for function arguments, locals, upvalues and some temporaries
    ///
    /// 0. generate for native modules
    /// 1. generate for all modules
    pub typeInfoLevel: c_int,

    /// Determines the degree to which converage information should be included into the bytecode
    pub coverageLevel: c_int,

    /// Global library to construct vectors; disabled by default
    pub vectorLib: *const c_char,

    /// Global builtin to construct vectors; disabled by default
    pub vectorCtor: *const c_char,

    /// Vector typename for type tables; disabled by default
    pub vectorType: *const c_char,

    /// `NULL`-terminated array of globals that are mutable; disables import optimizations for fields accessed through these
    pub mutableGlobals: *const *const c_char,

    /// `NULL`-terminated array of userdata types which will be included in the type information
    pub userdataTypes: *const *const c_char,

    /// null-terminated array of globals which act as libraries and have members with known type and/or constant value
    /// when an import of one of these libraries is accessed, library_member_type_callback and library_member_constant_callback below will be called to receive that information
    pub librariesWithKnownMembers: *const *const c_char,
    pub libraryMemberTypeCallback: Option<LuauLibraryMemberTypeCallback>,
    pub libraryMemberConstantCallback: Option<LuauLibraryMemberConstantCallback>,

    /// `NULL`-terminated array of builtins which will not be compiled into a fastcall ("name", "lib.name")
    pub disabledBuiltins: *const *const c_char,
}

impl Default for LuauCompileOptions {
    fn default() -> Self {
        Self {
            optimizationLevel: 1,
            debugLevel: 1,
            typeInfoLevel: 0,
            coverageLevel: 0,
            vectorLib: ptr::null(),
            vectorCtor: ptr::null(),
            vectorType: ptr::null(),
            mutableGlobals: ptr::null(),
            userdataTypes: ptr::null(),
            librariesWithKnownMembers: null(),
            libraryMemberTypeCallback: None,
            libraryMemberConstantCallback: None,
            disabledBuiltins: null(),
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
        options: *mut LuauCompileOptions,
        outsize: *mut usize,
    ) -> *mut c_char;
}

extern "C-unwind" {
    /// Sets a constant nil
    pub fn luau_set_compile_constant_nil(constant: LuauCompilerConstant);
    /// Sets a constant boolean
    pub fn luau_set_compile_constant_boolean(constant: LuauCompilerConstant, b: c_int);
    /// Sets a constant number
    pub fn luau_set_compile_constant_number(constant: LuauCompilerConstant, n: c_double);
    /// Sets a constant vector
    ///
    /// Vector component 'w' is not visible to VM runtime configured with LUA_VECTOR_SIZE == 3, but can affect constant folding during compilation
    pub fn luau_set_compile_constant_vector(
        constant: LuauCompilerConstant,
        x: c_float,
        y: c_float,
        z: c_float,
        w: c_float,
    );
    /// String storage must outlive the invocation of 'luau_compile' which used the callback
    pub fn luau_set_compile_constant_string(
        constant: LuauCompilerConstant,
        s: *const c_char,
        l: usize,
    );
}

extern "C" {
    #[link_name = "free"]
    pub fn cstdlib_free(ptr: *mut c_void);
}
