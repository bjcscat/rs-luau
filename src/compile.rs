use std::{ffi::c_int, fs, path::Path, ptr::null};

use crate::{ffi::luaucode, luau_compile, LuauCompileOptions};

pub struct Compiler {
    options: LuauCompileOptions,
}

impl Compiler {
    pub fn compile(&mut self, source: impl AsRef<[u8]>) -> Box<[u8]> {
        let source = source.as_ref();
        unsafe {
            let mut len = 0;
            let data = luau_compile(
                source.as_ptr() as _,
                source.len(),
                &raw mut self.options,
                &raw mut len,
            );
            luaucode::cstdlib_free(data as _);
            panic!()
        }
    }
}

pub struct CompilerBuilder(LuauCompileOptions);

impl CompilerBuilder {
    pub fn optimization_level(mut self, level: c_int) -> Self {
        self.0.optimization_level = level;
        self
    }

    pub fn build(self) -> Compiler {
        Compiler { options: self.0 }
    }
}

impl Default for CompilerBuilder {
    fn default() -> Self {
        Self(LuauCompileOptions {
            optimization_level: 1,
            debug_level: 1,
            type_info_level: 0,
            coverage_level: 0,
            vector_lib: null(),
            vector_ctor: null(),
            vector_type: null(),
            mutable_globals: null(),
            userdata_types: null(),
        })
    }
}
