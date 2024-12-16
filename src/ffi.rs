#![allow(dead_code, unused, clippy::missing_safety_doc)]

use std::ffi::c_int;

const fn const_parse_int(src: &'static str) -> i32 {
    match c_int::from_str_radix(src, 10) {
        Ok(v) => v,
        _ => panic!("Invalid env value passed to luaconf")
    }
}

pub mod luauconf {
    use std::ffi::c_int;

    use super::const_parse_int;

    /// LUA_IDSIZE gives the maximum size for the description of the source
    pub const LUA_IDSIZE: c_int = const_parse_int(env!("LUA_IDSIZE"));

    /// LUA_MINSTACK is the guaranteed number of Luau stack slots available to a C function
    pub const LUA_MINSTACK: c_int = const_parse_int(env!("LUA_MINSTACK"));

    /// LUAI_MAXCSTACK limits the number of Luau stack slots that a C function can use
    pub const LUAI_MAXCSTACK: c_int = const_parse_int(env!("LUAI_MAXCSTACK"));

    /// LUAI_MAXCALLS limits the number of nested calls
    pub const LUAI_MAXCALLS: c_int = const_parse_int(env!("LUAI_MAXCALLS"));

    /// LUAI_MAXCCALLS is the maximum depth for nested C calls; this limit depends on native stack size
    pub const LUAI_MAXCCALLS: c_int = const_parse_int(env!("LUAI_MAXCCALLS"));

    /// buffer size used for on-stack string operations; this limit depends on native stack size
    pub const LUA_BUFFERSIZE: c_int = const_parse_int(env!("LUA_BUFFERSIZE"));

    /// number of valid Luau userdata tags
    pub const LUA_UTAG_LIMIT: c_int = const_parse_int(env!("LUA_UTAG_LIMIT"));

    /// number of valid Luau lightuserdata tags
    pub const LUA_LUTAG_LIMIT: c_int = const_parse_int(env!("LUA_LUTAG_LIMIT"));

    /// upper bound for number of size classes used by page allocator
    pub const LUA_SIZECLASSES: c_int = const_parse_int(env!("LUA_SIZECLASSES"));

    /// available number of separate memory categories
    pub const LUA_MEMORY_CATEGORIES: c_int = const_parse_int(env!("LUA_MEMORY_CATEGORIES"));

    /// minimum size for the string table (must be power of 2)
    pub const LUA_MINSTRTABSIZE: c_int = const_parse_int(env!("LUA_MINSTRTABSIZE"));

    /// maximum number of captures supported by pattern matching
    pub const LUA_MAXCAPTURES: c_int = const_parse_int(env!("LUA_MAXCAPTURES"));

    /// the size of native Luau vectors
    pub const LUA_VECTOR_SIZE: c_int = const_parse_int(env!("LUA_VECTOR_SIZE"));
}

pub mod luau;
pub mod luaulib;
pub mod lauxlib;
#[cfg(feature="compiler")]
pub mod luaucode;
#[cfg(feature="codegen")]
pub mod luaucodegen;

#[allow(dead_code, unused)]
pub mod prelude {
    pub use super::luau::*;
    pub use super::lauxlib::*;
    pub use super::luaulib::*;
    #[cfg(feature="compiler")]
    pub use super::luaucode::*;
    #[cfg(feature="codegen")]
    pub use super::luaucodegen::*;
}
