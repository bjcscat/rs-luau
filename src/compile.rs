use core::str;
use std::{cell::UnsafeCell, ffi::c_int};

use crate::{cstdlib_free, luau_compile, LuauCompileOptions};

pub struct Compiler {
    _vector_lib: Option<Box<[u8]>>,
    _vector_ctor: Option<Box<[u8]>>,
    _vector_type: Option<Box<[u8]>>,
    _mutable_globals: Option<Vec<Box<[u8]>>>,
    _userdata_types: Option<Vec<Box<[u8]>>>,
    options: UnsafeCell<LuauCompileOptions>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            _vector_lib: None,
            _vector_ctor: None,
            _vector_type: None,
            _mutable_globals: None,
            _userdata_types: None,
            options: UnsafeCell::new(LuauCompileOptions::default()),
        }
    }

    fn options(&mut self) -> &mut LuauCompileOptions {
        self.options.get_mut()
    }

    /// Sets the optimization level for the compiler
    pub fn optimization_level(&mut self, level: c_int) -> &mut Self {
        self.options().optimization_level = level;

        self
    }

    /// Sets the debug level for the compiler
    pub fn debug_level(&mut self, level: c_int) -> &mut Self {
        self.options().debug_level = level;

        self
    }

    /// Sets the type info level for the compiler
    pub fn type_info_level(&mut self, level: c_int) -> &mut Self {
        self.options().type_info_level = level;

        self
    }

    /// Sets the coverage level for the compiler
    pub fn coverage_level(&mut self, level: c_int) -> &mut Self {
        self.options().coverage_level = level;

        self
    }

    /// Sets the vector library ident for the compiler
    pub fn vector_lib(&mut self, lib: impl AsRef<[u8]>) -> &mut Self {
        let lib: Box<[u8]> = Box::from(lib.as_ref());

        self.options().vector_lib = lib.as_ptr() as _;
        self._vector_lib = Some(lib);

        self
    }

    /// Sets the vector constructor ident for the compiler
    pub fn vector_ctor(&mut self, ctor: impl AsRef<[u8]>) -> &mut Self {
        let ctor: Box<[u8]> = Box::from(ctor.as_ref());

        self.options().vector_ctor = ctor.as_ptr() as _;
        self._vector_ctor = Some(ctor);

        self
    }

    /// Sets the vector type ident for the compiler
    pub fn vector_type(&mut self, vec_type: impl AsRef<[u8]>) -> &mut Self {
        let vector_type: Box<[u8]> = Box::from(vec_type.as_ref());

        self.options().vector_type = vector_type.as_ptr() as _;
        self._vector_type = Some(vector_type);

        self
    }

    /// Sets the mutable globals for the compiler
    pub fn mutable_globals<T: AsRef<[u8]>>(&mut self, lib: impl AsRef<[T]>) -> &mut Self {
        let mut vector: Vec<Box<[u8]>> = Vec::new();
        let mut pointer_vectors: Vec<*const u8> = Vec::new();

        for t in lib.as_ref() {
            let boxed: Box<[u8]> = Box::from(t.as_ref());
            pointer_vectors.push(boxed.as_ptr());
            vector.push(boxed);
        }

        self.options().mutable_globals = pointer_vectors.as_ptr() as *const *const _;
        self._mutable_globals = Some(vector);

        self
    }

    /// Sets the userdata types for the compiler
    pub fn userdata_types<T: AsRef<[u8]>>(&mut self, types: impl AsRef<[T]>) -> &mut Self {
        let mut vector: Vec<Box<[u8]>> = Vec::new();
        let mut pointer_vectors: Vec<*const u8> = Vec::new();

        for t in types.as_ref() {
            let boxed: Box<[u8]> = Box::from(t.as_ref());
            pointer_vectors.push(boxed.as_ptr());
            vector.push(boxed);
        }

        self.options().userdata_types = pointer_vectors.as_ptr() as *const *const _;
        self._userdata_types = Some(vector);

        self
    }

    pub fn compile(&self, source: impl AsRef<[u8]>) -> CompilerResult {
        let source = source.as_ref();
        unsafe {
            let mut len = 0;
            let bytecode = luau_compile(
                source.as_ptr() as _,
                source.len(),
                self.options.get(),
                &raw mut len,
            ) as *const i8; // explicit conversion needed to compile on android

            CompilerResult { bytecode, len }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompilerResult {
    bytecode: *const i8,
    len: usize,
}

impl CompilerResult {
    // not technically unsafe
    fn bytecode_unchecked(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bytecode as _, self.len) }
    }

    pub fn bytecode(&self) -> Option<&[u8]> {
        if self.is_err() {
            None
        } else {
            Some(self.bytecode_unchecked())
        }
    }

    pub fn error(&self) -> Option<&str> {
        if self.is_ok() {
            None
        } else {
            unsafe {
                Some(
                    str::from_utf8(std::slice::from_raw_parts(
                        self.bytecode.add(1) as _,
                        self.len - 1,
                    ))
                    .expect("Luau error was not valid UTF-8"),
                )
            }
        }
    }

    /// Returns true if the compiler result is an error
    pub fn is_err(&self) -> bool {
        unsafe { !self.bytecode.is_null() && self.bytecode.read() == 0 }
    }

    /// Returns true if the compiler result is not an error
    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }
}

impl Drop for CompilerResult {
    fn drop(&mut self) {
        unsafe {
            cstdlib_free(self.bytecode as _);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Luau;

    use super::Compiler;

    #[test]
    fn compiler_success() {
        let compiler = Compiler::new();

        // has an effect so cant be optimized out entirely
        let result = compiler.compile("v()");

        assert!(result.is_ok(), "Expected result to be a success");
        assert!(
            result.bytecode().is_some(),
            "Expected resultant bytecode to be some"
        );
        assert!(
            result.bytecode().is_some_and(|v| !v.is_empty()),
            "Expected resultant bytecode to be non-empty"
        );

        let luau = Luau::default();

        let load_result = luau.load(None, result.bytecode().unwrap(), 0);

        assert_eq!(load_result, Ok(()));
    }

    #[test]
    fn compiler_error() {
        let compiler = Compiler::new();

        // will always be an error per an RFC
        let result = compiler.compile("$");

        assert!(
            result.is_err(),
            "Expected the compiler result to be an error"
        );

        assert!(
            result.bytecode().is_none(),
            "Expected the bytecode to be none"
        );
        assert!(
            result.error().is_some(),
            "Expected the compiler result output a string"
        );
    }
}
