use std::{
    ffi::{c_char, c_int, CString},
    ptr::null,
};

use crate::{
    cstdlib_free, luau_compile, LuauCompileOptions, LuauLibraryMemberConstantCallback,
    LuauLibraryMemberTypeCallback,
};

#[derive(Debug, Clone)]
pub struct CompilerLibraries {
    libraries: Vec<String>,
    member_type_callback: LuauLibraryMemberTypeCallback,
    member_constant_callback: LuauLibraryMemberConstantCallback,
}

impl CompilerLibraries {
    pub fn new(
        libraries: Vec<String>,
        member_type_callback: LuauLibraryMemberTypeCallback,
        member_constant_callback: LuauLibraryMemberConstantCallback,
    ) -> Self {
        Self {
            libraries,
            member_type_callback,
            member_constant_callback,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Compiler {
    optimization_level: u8,
    debug_level: u8,
    type_info_level: u8,
    coverage_level: u8,
    vector_lib: Option<String>,
    vector_ctor: Option<String>,
    vector_type: Option<String>,
    mutable_globals: Vec<String>,
    userdata_types: Vec<String>,
    disabled_builtins: Vec<String>,
    libs: Option<CompilerLibraries>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            optimization_level: 1,
            debug_level: 1,
            type_info_level: 0,
            coverage_level: 0,
            vector_lib: None,
            vector_ctor: None,
            vector_type: None,
            mutable_globals: Vec::new(),
            userdata_types: Vec::new(),
            disabled_builtins: Vec::new(),
            libs: None,
        }
    }
    /// Sets Luau compiler optimization level.
    ///
    /// Possible values:
    /// * 0 - no optimization
    /// * 1 - baseline optimization level that doesn't prevent debuggability (default)
    /// * 2 - includes optimizations that harm debuggability such as inlining
    #[must_use]
    pub const fn set_optimization_level(mut self, level: u8) -> Self {
        self.optimization_level = level;
        self
    }

    /// Sets Luau compiler debug level.
    ///
    /// Possible values:
    /// * 0 - no debugging support
    /// * 1 - line info & function names only; sufficient for backtraces (default)
    /// * 2 - full debug info with local & upvalue names; necessary for debugger
    #[must_use]
    pub const fn set_debug_level(mut self, level: u8) -> Self {
        self.debug_level = level;
        self
    }

    /// Sets Luau type information level used to guide native code generation decisions.
    ///
    /// Possible values:
    /// * 0 - generate for native modules (default)
    /// * 1 - generate for all modules
    pub const fn set_type_info_level(mut self, level: u8) -> Self {
        self.type_info_level = level;
        self
    }

    /// Sets Luau compiler code coverage level.
    ///
    /// Possible values:
    /// * 0 - no code coverage support (default)
    /// * 1 - statement coverage
    /// * 2 - statement and expression coverage (verbose)
    #[must_use]
    pub const fn set_coverage_level(mut self, level: u8) -> Self {
        self.coverage_level = level;
        self
    }

    #[must_use]
    pub fn set_vector_lib(mut self, lib: impl Into<String>) -> Self {
        self.vector_lib = Some(lib.into());
        self
    }

    #[must_use]
    pub fn set_vector_ctor(mut self, ctor: impl Into<String>) -> Self {
        self.vector_ctor = Some(ctor.into());
        self
    }

    #[must_use]
    pub fn set_vector_type(mut self, r#type: impl Into<String>) -> Self {
        self.vector_type = Some(r#type.into());
        self
    }

    /// Sets a list of globals that are mutable.
    ///
    /// It disables the import optimization for fields accessed through these.
    #[must_use]
    pub fn set_mutable_globals(mut self, globals: Vec<String>) -> Self {
        self.mutable_globals = globals;
        self
    }

    /// Sets a list of userdata types that will be included in the type information.
    #[must_use]
    pub fn set_userdata_types(mut self, types: Vec<String>) -> Self {
        self.userdata_types = types;
        self
    }

    /// Sets a list of disabled builtin libs or functions like tonumber or math.abs
    pub fn set_disabled_builtins(mut self, libs: Vec<String>) -> Self {
        self.disabled_builtins = libs;
        self
    }

    pub fn set_libraries(&mut self, libraries: CompilerLibraries) -> &mut Self {
        let mut pointer_vec = Vec::with_capacity(libraries.libraries.len());

        for v in &libraries.libraries {
            pointer_vec.push(v.as_ptr());
        }

        self.libs = Some(libraries);

        self
    }

    #[must_use]
    pub fn compile(&self, source: impl AsRef<[u8]>) -> CompilerResult {
        let vector_lib = self.vector_lib.clone();
        let vector_lib = vector_lib.and_then(|lib| CString::new(lib).ok());
        let vector_lib = vector_lib.as_ref();
        let vector_ctor = self.vector_ctor.clone();
        let vector_ctor = vector_ctor.and_then(|ctor| CString::new(ctor).ok());
        let vector_ctor = vector_ctor.as_ref();
        let vector_type = self.vector_type.clone();
        let vector_type = vector_type.and_then(|t| CString::new(t).ok());
        let vector_type = vector_type.as_ref();

        macro_rules! vec2cstring_ptr {
            ($name:ident, $name_ptr:ident) => {
                let $name = self
                    .$name
                    .iter()
                    .map(|name| CString::new(name.clone()).ok())
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default();
                let mut $name = $name.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
                let mut $name_ptr = null();
                if !$name.is_empty() {
                    $name.push(null());
                    $name_ptr = $name.as_ptr();
                }
            };
        }

        vec2cstring_ptr!(mutable_globals, mutable_globals_ptr);
        vec2cstring_ptr!(userdata_types, userdata_types_ptr);
        vec2cstring_ptr!(disabled_builtins, disabled_builtins_ptr);

        let known_members_vec = self.libs.clone().map(|v| {
            v.libraries
                .into_iter()
                .map(|s| CString::new(s).expect("Known members should not contain null byte"))
                .collect::<Vec<_>>()
        });

        let mut known_members_vec_pointer = known_members_vec.map_or_else(
            || vec![null()],
            |v| v.iter().map(|c| c.as_ptr()).collect::<Vec<_>>(),
        );

        known_members_vec_pointer.push(null());
            
        unsafe {
            let mut options = LuauCompileOptions {
                optimizationLevel: self.optimization_level as c_int,
                debugLevel: self.debug_level as c_int,
                typeInfoLevel: self.type_info_level as c_int,
                coverageLevel: self.coverage_level as c_int,
                vectorLib: vector_lib.map_or(null(), |s| s.as_ptr()),
                vectorCtor: vector_ctor.map_or(null(), |s| s.as_ptr()),
                vectorType: vector_type.map_or(null(), |s| s.as_ptr()),
                mutableGlobals: mutable_globals_ptr,
                userdataTypes: userdata_types_ptr,
                librariesWithKnownMembers: known_members_vec_pointer.as_ptr(),
                libraryMemberTypeCallback: self.libs.clone().map(|v| v.member_type_callback),
                libraryMemberConstantCallback: self
                    .libs
                    .clone()
                    .map(|v| v.member_constant_callback),
                disabledBuiltins: disabled_builtins_ptr,
            };

            let source = source.as_ref();
            let mut len: usize = 0;

            let bytecode = luau_compile(
                source.as_ptr() as _,
                source.len(),
                &raw mut options,
                &raw mut len,
            );

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
            .set_mutable_globals(vec!["a".to_string()])
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
            vec!["test".to_string()],
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
            vec!["test".to_string()],
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
