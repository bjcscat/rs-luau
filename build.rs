use cmake::Config;

/// Exposes a Luau cfg value into the C/C++ compiler and optionally permits implementor to override some fields with an env var
macro_rules! define_lua_cfg {
    ([no_override] $config:ident, $c_macro:literal, $value:expr) => {
        let value = $value;
        $config.cflag(format!("-D{}={}", $c_macro, value));
        $config.cxxflag(format!("-D{}={}", $c_macro, value));
        println!("cargo::rustc-env={}={}", $c_macro, value);
    };

    ($config:ident, $c_macro:literal, $src:literal) => {
        let val = std::env::var($c_macro).unwrap_or(($src).to_string());
        define_lua_cfg!([no_override] $config, $c_macro, val)
    };
}

// most of these can be overriden by the implementor setting env vars
fn do_cfg(config: &mut Config) {
    // LUA_IDSIZE gives the maximum size for the description of the source
    define_lua_cfg!(config, "LUA_IDSIZE", "256");

    // LUA_MINSTACK is the guaranteed number of Luau stack slots available to a C function
    define_lua_cfg!(config, "LUA_MINSTACK", "20");

    // LUAI_MAXCSTACK limits the number of Luau stack slots that a C function can use
    define_lua_cfg!(config, "LUAI_MAXCSTACK", "8000");

    // LUAI_MAXCALLS limits the number of nested calls
    define_lua_cfg!(config, "LUAI_MAXCALLS", "20000");

    // LUAI_MAXCCALLS is the maximum depth for nested C calls; this limit depends on native stack size
    define_lua_cfg!(config, "LUAI_MAXCCALLS", "200");

    // buffer size used for on-stack string operations; this limit depends on native stack size
    define_lua_cfg!(config, "LUA_BUFFERSIZE", "512");

    // number of valid Luau userdata tags
    define_lua_cfg!(config, "LUA_UTAG_LIMIT", "128");

    // number of valid Luau lightuserdata tags
    define_lua_cfg!(config, "LUA_LUTAG_LIMIT", "128");

    // upper bound for number of size classes used by page allocator
    define_lua_cfg!(config, "LUA_SIZECLASSES", "40");

    // available number of separate memory categories
    define_lua_cfg!(config, "LUA_MEMORY_CATEGORIES", "256");

    // minimum size for the string table (must be power of 2)
    define_lua_cfg!(config, "LUA_MINSTRTABSIZE", "32");

    // maximum number of captures supported by pattern matching
    define_lua_cfg!(config, "LUA_MAXCAPTURES", "32");

    #[cfg(not(feature="luau_vector4"))]
    define_lua_cfg!(config, "LUA_VECTOR_SIZE", "3");

    #[cfg(feature="luau_vector4")]
    define_lua_cfg!(config, "LUA_VECTOR_SIZE", "4");
}

fn main() {
    let mut config = cmake::Config::new("luau");

    config
        .define("LUAU_BUILD_CLI", "OFF")
        .define("LUAU_BUILD_TESTS", "OFF")
        .define("LUAU_STATIC_CRT", "ON")
        .define("LUAU_EXTERN_C", "ON")
        .define("LUAU_ENABLE_ASSERT", "ON")
        .profile(if cfg!(debug_assertions) {
            "RelWithDebInfo"
        } else {
            "Release"
        })
        .no_build_target(true);

    do_cfg(&mut config);

    // unwinding in order for exceptions but cmake-rs wipes this by default
    #[cfg(target_os = "windows")]
    config.cxxflag("/EHsc");

    let destination = config.build();

    #[cfg(not(target_os = "windows"))]
    println!(
        "cargo:rustc-link-search=native={}/build",
        destination.display()
    );

    // Windows is once again special and outputs libs in even more subdirs
    #[cfg(target_os = "windows")]
    {
        #[cfg(debug_assertions)]
        println!(
            "cargo:rustc-link-search=native={}/build/RelWithDebInfo",
            destination.display()
        );
        #[cfg(not(debug_assertions))]
        println!(
            "cargo:rustc-link-search=native={}/build/Release",
            destination.display()
        );
    }

    println!("cargo:rustc-link-lib=static=Luau.VM");

    #[cfg(feature = "compiler")]
    println!("cargo:rustc-link-lib=static=Luau.Compiler");
    #[cfg(feature = "compiler")]
    println!("cargo:rustc-link-lib=static=Luau.Ast");
    // println!("cargo:rustc-link-lib=static=Luau.Analysis");

    #[cfg(feature = "codegen")]
    println!("cargo:rustc-link-lib=static=Luau.CodeGen");

    // link to C++ stdlib, unless we're on windows, which is special
    #[cfg(not(target_os = "windows"))]
    println!("cargo:rustc-link-lib=stdc++");
}
