use std::{
    cell::UnsafeCell,
    ffi::{c_char, c_int, CStr, CString},
};

use crate::{
    cstdlib_free, luau_compile, LuauCompileOptions, LuauLibraryMemberConstantCallback,
    LuauLibraryMemberTypeCallback,
};

#[derive(Debug, Clone)]
pub struct CompilerLibraries {
    libraries: Vec<CString>,
    member_type_callback: LuauLibraryMemberTypeCallback,
    member_constant_callback: LuauLibraryMemberConstantCallback,
}

impl CompilerLibraries {
    pub fn new<T: AsRef<[u8]>>(
        libraries: impl AsRef<[T]>,
        member_type_callback: LuauLibraryMemberTypeCallback,
        member_constant_callback: LuauLibraryMemberConstantCallback,
    ) -> Self {
        let libraries = libraries.as_ref();
        let mut cstring_vec = Vec::with_capacity(libraries.len());

        for v in libraries {
            cstring_vec.push(
                CString::new(v.as_ref()).expect("Library names should not contain null bytes"),
            );
        }

        Self {
            libraries: cstring_vec,
            member_type_callback,
            member_constant_callback,
        }
    }
}

pub struct Compiler {
    _vector_lib: Option<CString>,
    _vector_ctor: Option<CString>,
    _vector_type: Option<CString>,
    _mutable_globals: Option<Vec<CString>>,
    _userdata_types: Option<Vec<CString>>,
    _libraries_with_known_members: Option<Vec<CString>>,
    _disabled_builtins: Option<Vec<CString>>,
    _libs: Option<CompilerLibraries>,
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
            _libraries_with_known_members: None,
            _disabled_builtins: None,
            _libs: None,
            options: UnsafeCell::new(LuauCompileOptions::default()),
        }
    }

    /// Sets the optimization level for the compiler
    pub fn set_optimization_level(&mut self, level: c_int) -> &mut Self {
        self.options.get_mut().optimization_level = level;

        self
    }

    /// Sets the debug level for the compiler
    pub fn set_debug_level(&mut self, level: c_int) -> &mut Self {
        self.options.get_mut().debug_level = level;

        self
    }

    /// Sets the type info level for the compiler
    pub fn set_type_info_level(&mut self, level: c_int) -> &mut Self {
        self.options.get_mut().type_info_level = level;

        self
    }

    /// Sets the coverage level for the compiler
    pub fn set_coverage_level(&mut self, level: c_int) -> &mut Self {
        self.options.get_mut().coverage_level = level;

        self
    }

    /// Sets the vector library ident for the compiler
    pub fn set_vector_lib(&mut self, lib: impl AsRef<[u8]>) -> &mut Self {
        let lib =
            CString::new(lib.as_ref()).expect("Compiler arguments may not contain a null byte");

        self.options.get_mut().vector_lib = lib.as_ptr() as _;
        self._vector_lib = Some(lib);

        self
    }

    /// Sets the vector constructor ident for the compiler
    pub fn set_vector_ctor(&mut self, ctor: impl AsRef<[u8]>) -> &mut Self {
        let ctor =
            CString::new(ctor.as_ref()).expect("Compiler arguments may not contain a null byte");

        self.options.get_mut().vector_ctor = ctor.as_ptr() as _;
        self._vector_ctor = Some(ctor);

        self
    }

    /// Sets the vector type ident for the compiler
    pub fn set_vector_type(&mut self, vec_type: impl AsRef<[u8]>) -> &mut Self {
        let vector_type = CString::new(vec_type.as_ref())
            .expect("Compiler arguments may not contain a null byte");

        self.options.get_mut().vector_type = vector_type.as_ptr() as _;
        self._vector_type = Some(vector_type);

        self
    }

    /// Sets the mutable globals for the compiler
    pub fn set_mutable_globals<T: AsRef<[u8]>>(&mut self, lib: impl AsRef<[T]>) -> &mut Self {
        let mut vector = Vec::new();
        let mut pointer_vectors = Vec::new();

        for t in lib.as_ref() {
            let string =
                CString::new(t.as_ref()).expect("Compiler arguments may not contain a null byte");
            pointer_vectors.push(string.as_ptr());
            vector.push(string);
        }

        self.options.get_mut().mutable_globals = pointer_vectors.as_ptr();
        self._mutable_globals = Some(vector);

        self
    }

    /// Sets the userdata types for the compiler
    pub fn set_userdata_types<T: AsRef<CStr>>(&mut self, types: impl AsRef<[T]>) -> &mut Self {
        let mut vector: Vec<CString> = Vec::new();
        let mut pointer_vectors: Vec<*const c_char> = Vec::new();

        for t in types.as_ref() {
            let boxed = CString::from(t.as_ref());
            pointer_vectors.push(boxed.as_ptr());
            vector.push(boxed);
        }

        self.options.get_mut().userdata_types = pointer_vectors.as_ptr();
        self._userdata_types = Some(vector);

        self
    }

    pub fn set_libraries(&mut self, libraries: CompilerLibraries) -> &mut Self {
        let mut pointer_vec = Vec::with_capacity(libraries.libraries.len());

        for v in &libraries.libraries {
            pointer_vec.push(v.as_ptr());
        }

        self._libs = Some(libraries);
        self.options.get_mut().libraries_with_known_members = pointer_vec.as_ptr();

        self
    }
    
    #[must_use]
    pub fn compile(&self, source: impl AsRef<[u8]>) -> CompilerResult {
        let source = source.as_ref();
        unsafe {
            let mut len = 0;
            let bytecode = luau_compile(
                source.as_ptr() as _,
                source.len(),
                self.options.get(),
                &raw mut len,
            );

            CompilerResult { bytecode, len }
        }
    }
}

impl Clone for Compiler {
    fn clone(&self) -> Self {
        let mut options = Self::new();

        // not aliasing a mutable reference
        let original_options = unsafe { self.options.get().as_ref() }.unwrap();

        options.set_optimization_level(original_options.optimization_level);
        options.set_debug_level(original_options.debug_level);
        options.set_type_info_level(original_options.type_info_level);
        options.set_coverage_level(original_options.coverage_level);

        if let Some(vector_lib) = &self._vector_lib {
            options.set_vector_lib(vector_lib.as_bytes());
        }
        if let Some(vector_ctor) = &self._vector_ctor {
            options.set_vector_ctor(vector_ctor.as_bytes());
        }
        if let Some(vector_type) = &self._vector_type {
            options.set_vector_type(vector_type.as_bytes());
        }
        if let Some(mutable_globals) = &self._mutable_globals {
            let cstring_vec = mutable_globals.clone();
            let pointer_vec = Vec::with_capacity(cstring_vec.len());
            options._mutable_globals = Some(cstring_vec);
            options.options.get_mut().mutable_globals = pointer_vec.as_ptr();
        }

        if let Some(userdata_types) = &self._userdata_types {
            let cstring_vec = userdata_types.clone();
            let pointer_vec = Vec::with_capacity(cstring_vec.len());
            options._userdata_types = Some(cstring_vec);
            options.options.get_mut().userdata_types = pointer_vec.as_ptr();
        }

        if let Some(disabled_builtins) = &self._disabled_builtins {
            let cstring_vec = disabled_builtins.clone();
            let pointer_vec = Vec::with_capacity(cstring_vec.len());
            options._disabled_builtins = Some(cstring_vec);
            options.options.get_mut().disabled_builtins = pointer_vec.as_ptr();
        }

        if let Some(libs) = self._libs.clone() {
            options.set_libraries(libs);
        }

        options
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompilerResult {
    bytecode: *const c_char,
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
                    std::str::from_utf8(std::slice::from_raw_parts(
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
    use std::ffi::c_char;

    use crate::{Luau, LuauBytecodeType, LuauCompilerConstant};

    use super::{Compiler, CompilerLibraries};


    unsafe extern "C-unwind" fn member_type_callback(
        _: *const c_char,
        _: *const c_char,
    ) -> LuauBytecodeType {
        LuauBytecodeType::LBC_TYPE_BOOLEAN
    }

    unsafe extern "C-unwind" fn member_constant_callback(
        _: *const c_char,
        _: *const c_char,
        _: LuauCompilerConstant,
    ) {
    }

    #[test]
    fn compiler_success() {
        let mut compiler = Compiler::new();

        // has an effect so cant be optimized out entirely
        let result = compiler
            .set_optimization_level(2)
            .set_coverage_level(1)
            .set_mutable_globals(["a"])
            .compile("v()");

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
    fn libs() {
        let mut compiler = Compiler::new();

        compiler.set_libraries(CompilerLibraries::new(
            ["test"],
            member_type_callback,
            member_constant_callback,
        ));

        let compiler_result = compiler.compile("local a = test.test");

        assert!(compiler_result.is_ok(), "Expected compiler to succeed");
    }

    #[test]
    fn cloned_compiler() {
        let mut compiler = {
            let original_compiler = Compiler::new();
            original_compiler.clone()
        };

        compiler.set_libraries(CompilerLibraries::new(
            ["test"],
            member_type_callback,
            member_constant_callback,
        ));

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
