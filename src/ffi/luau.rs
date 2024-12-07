use std::{
    ffi::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_void},
    marker::{PhantomData, PhantomPinned},
    ptr::null_mut,
};

use super::luauconf::LUAI_MAXCSTACK;

pub const LUA_MULTRET: c_int = -1;
pub const LUA_REGISTRYINDEX: c_int = -LUAI_MAXCSTACK - 2000;
pub const LUA_ENVIRONINDEX: c_int = -LUAI_MAXCSTACK - 2001;
pub const LUA_GLOBALSINDEX: c_int = -LUAI_MAXCSTACK - 2002;

#[inline]
pub const fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}

#[inline]
pub const fn lua_ispseudo(i: c_int) -> bool {
    i <= LUA_REGISTRYINDEX
}

// thread status; 0 is OK
#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum LuauStatus {
    /// OK status
    LUA_OK = 0,
    /// Yielded
    LUA_YIELD,
    /// Errored on a runtime error
    LUA_ERRRUN,
    #[deprecated = "legacy error code, preserved for compatibility"]
    LUA_ERRSYNTAX,
    /// Errored on a memory allocation failure
    LUA_ERRMEM,
    /// Errored in error handling
    LUA_ERRERR,
    /// Yielded on a breakpoint
    LUA_BREAK,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum CoroutineStatus {
    /// The coroutine is running
    LUA_CORUN = 0,
    /// The coroutine is suspsneded
    LUA_COSUS,
    /// The couroutine is 'normal' (resumed another coroutine)
    LUA_CONOR,
    /// The coroutine finished successfully
    LUA_COFIN,
    /// The coroutine finished with an error
    LUA_COERR,
}

/// A raw Luau state associated with a thread.
#[repr(C)]
pub struct _LuaState {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

#[repr(transparent)]
pub struct Tag(pub c_int);

/// Type for native C functions that can be passed to Luau.
pub type CFunction = unsafe extern "C-unwind" fn(*mut _LuaState) -> c_int;
/// Type for unrolling across the stack
pub type LuaContinuation = unsafe extern "C-unwind" fn(*mut _LuaState, c_int) -> c_int;
/// Type for Luau allocation functions
pub type LuaAlloc = unsafe extern "C-unwind" fn(
    ud: *mut c_void,
    ptr: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void;
/// Type for Luau dtor functions
pub type LuaDestructor = unsafe extern "C-unwind" fn(*mut _LuaState, *mut c_void);

/// Luau type value
#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd)]
#[allow(non_camel_case_types)]
pub enum LuauType {
    LUA_TNONE = -1,
    LUA_TNIL = 0,     // must be 0 due to lua_isnoneornil
    LUA_TBOOLEAN = 1, // must be 1 due to l_isfalse

    LUA_TLIGHTUSERDATA,
    LUA_TNUMBER,
    LUA_TVECTOR,

    LUA_TSTRING, // all types above this must be value types, all types below this must be GC types - see iscollectable

    LUA_TTABLE,
    LUA_TFUNCTION,
    LUA_TUSERDATA,
    LUA_TTHREAD,
    LUA_TBUFFER,

    // values below this line are used in GCObject tags but may never show up in TValue type tags
    LUA_TPROTO,
    LUA_TUPVAL,
    LUA_TDEADKEY,
}

pub const LUA_T_COUNT: c_int = LuauType::LUA_TPROTO as c_int;

/// Type of numbers in Luau
pub type LuaNumber = c_double;

/// Type for integer functions
pub type LuaInteger = c_int;

/// Unsigned integer type
pub type LuaUnsigned = c_uint;

extern "C-unwind" {
    /// Creates a new independent state and returns its main thread.
    /// Returns NULL if it cannot create the state (due to lack of memory).
    /// The argument f is the allocator function; Luau will do all memory allocation for this state through this function (see LuaAlloc).
    /// The second argument, ud, is an opaque pointer that Luau passes to the allocator in every call.
    pub fn lua_newstate(f: LuaAlloc, ud: *mut c_void) -> *mut _LuaState;

    /// Close all active to-be-closed variables in the main thread, release all objects in the given Luau state (calling the corresponding garbage-collection metamethods, if any), and frees all dynamic memory used by this state.
    ///
    /// On several platforms, you may not need to call this function, because all resources are naturally released when the host program ends.
    /// On the other hand, long-running programs that create multiple states, such as daemons or web servers, will probably need to close states as soon as they are not needed.
    pub fn lua_close(state: *mut _LuaState);

    /// Creates a new thread, pushes it on the stack, and returns a pointer to a lua_State that represents this new thread.
    /// The new thread returned by this function shares with the original thread its global environment, but has an independent execution stack.
    ///
    /// Threads are subject to garbage collection, like any Luau object.
    pub fn lua_newthread(state: *mut _LuaState) -> *mut _LuaState;

    /// Retrieves the main thread for a thread state
    pub fn lua_mainthread(state: *mut _LuaState) -> *mut _LuaState;

    /// Resets a thread, cleaning its call stack and closing all pending to-be-closed variables.
    /// In case of error, leaves the error object on the top of the stack.
    pub fn lua_resetthread(state: *mut _LuaState);

    /// Determines if a thread is reset
    pub fn lua_isthreadreset(state: *mut _LuaState) -> c_int;
}

extern "C-unwind" {
    /// Converts the acceptable index idx into an equivalent absolute index (that is, one that does not depend on the stack size).
    pub fn lua_absindex(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Returns the index of the top element in the stack.
    /// Because indices start at 1, this result is equal to the number of elements in the stack; in particular, 0 means an empty stack.
    pub fn lua_gettop(state: *mut _LuaState) -> c_int;

    /// Accepts any index, or 0, and sets the stack top to this index. If the new top is greater than the old one, then the new elements are filled with nil. If index is 0, then all stack elements are removed.
    pub fn lua_settop(state: *mut _LuaState, idx: c_int);

    /// Pushes a copy of the element at the given index onto the stack.
    pub fn lua_pushvalue(state: *mut _LuaState, idx: c_int);

    /// Removes the element at the given valid index, shifting down the elements above this index to fill the gap.
    pub fn lua_remove(state: *mut _LuaState, idx: c_int);

    /// Moves the top element into the given valid index, shifting up the elements above this index to open space.
    pub fn lua_insert(state: *mut _LuaState, idx: c_int);

    /// Moves the top element into the given valid index without shifting any element (therefore replacing the value at that given index), and then pops the top element.
    pub fn lua_replace(state: *mut _LuaState, idx: c_int);

    /// Ensures that the stack has space for at least n extra elements, that is, that you can safely push up to n values into it.
    /// Will throw if it cannot perform an allocation or will return false if would cause the stack to be greater than a fixed maximum size.
    /// This function never shrinks the stack; if the stack already has space for the extra elements, it is left unchanged.
    pub fn lua_checkstack(state: *mut _LuaState, sz: c_int) -> c_int;

    /// Same as lua_checkstack but allows for unlimited stack frames
    pub fn lua_rawcheckstack(state: *mut _LuaState, sz: c_int);
}

extern "C-unwind" {
    /// Exchange values between different threads of the same state.
    ///
    /// This function pops n values from the stack from, and pushes them onto the stack to.
    pub fn lua_xmove(from: *mut _LuaState, to: *mut _LuaState, idx: c_int);

    /// Exchange values between different threads of the same state.
    ///
    /// This function copies n values from the stack from, and pushes them onto the stack to.
    pub fn lua_xpush(from: *mut _LuaState, to: *mut _LuaState, idx: c_int);
}

extern "C-unwind" {
    /// Returns 1 if the value at the given index is a number or a string convertible to a number, and 0 otherwise.
    pub fn lua_isnumber(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Returns 1 if the value at the given index is a string or a number (which is always convertible to a string), and 0 otherwise.
    pub fn lua_isstring(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Returns 1 if the value at the given index is a C function, and 0 otherwise.
    pub fn lua_iscfunction(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Returns 1 if the value at the given index is a Luau function, and 0 otherwise.
    pub fn lua_isLfunction(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Returns 1 if the value at the given index is a userdata (either full or light), and 0 otherwise.
    pub fn lua_isuserdata(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Returns the type of the value in the given valid index, or LUA_TNONE for a non-valid but acceptable index.
    pub fn lua_type(state: *mut _LuaState, idx: c_int) -> LuauType;

    /// Returns the name of the type encoded by the value tp, which must be one the values returned by lua_type.
    pub fn lua_typename(state: *mut _LuaState, tp: LuauType) -> *const c_char;
}

extern "C-unwind" {
    /// Runs a equality check for two values at 2 indexes, may invoke metamethods
    pub fn lua_equal(state: *mut _LuaState, idx1: c_int, idx2: c_int) -> c_int;

    /// Runs a equality check for two values at 2 indexes, will not invoke metamethods
    pub fn lua_rawequal(state: *mut _LuaState, idx1: c_int, idx2: c_int) -> c_int;

    /// Runs a lessthan check for two values at 2 indexes, may invoke metamethods
    pub fn lua_lessthan(state: *mut _LuaState, idx1: c_int, idx2: c_int) -> c_int;
}

extern "C-unwind" {
    /// Converts the Luau value at the given index to the type LuaNumber.
    /// The Luau value must be a number or a string convertible to a number; otherwise returns 0.
    ///
    /// If isnum is not NULL, its referent is assigned a boolean value that indicates whether the operation succeeded.
    pub fn lua_tonumberx(state: *mut _LuaState, idx: c_int, isnum: *mut c_int) -> LuaNumber;

    /// Converts the Luau value at the given index to the signed integral type LuaInteger.
    /// The Luau value must be an integer, or a number or string convertible to an integer; otherwise, lua_tointegerx returns 0.
    ///
    /// If isnum is not NULL, its referent is assigned a boolean value that indicates whether the operation succeeded.
    pub fn lua_tointegerx(state: *mut _LuaState, idx: c_int, isnum: *mut c_int) -> LuaInteger;

    /// Converts the Luau value at the given index to the unsigned integral type LuaUnsigned.
    /// The Luau value must be an integer, or a number or string convertible to an integer; otherwise, lua_tounsignedx returns 0.
    ///
    /// If isnum is not NULL, its referent is assigned a boolean value that indicates whether the operation succeeded.
    pub fn lua_tounsignedx(state: *mut _LuaState, idx: c_int, isnum: *mut c_int) -> LuaUnsigned;

    /// Gets the Luau value's vector pointer.
    ///
    /// If the value is not a vector then will return NULL
    pub fn lua_tovector(state: *mut _LuaState, idx: c_int) -> *const c_float;

    /// Converts the Luau value at the given index to a C boolean value (0 or 1).
    /// Like all tests in Luau, lua_toboolean returns true for any Luau value different from false and nil; otherwise it returns false. (If you want to accept only actual boolean values, use lua_isboolean to test the value's type.)
    pub fn lua_toboolean(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Converts the Luau value at the given index to a C string. If len is not NULL, it sets *len with the string length.
    /// The Luau value must be a string or a number; otherwise, the function returns NULL.
    /// If the value is a number, then lua_tolstring also changes the actual value in the stack to a string. (This change confuses lua_next when lua_tolstring is applied to keys during a table traversal.)
    /// lua_tolstring returns a pointer to a string inside the Luau state. This string always has a zero ('\0') after its last character (as in C), but can contain other zeros in its body.
    pub fn lua_tolstring(state: *mut _LuaState, idx: c_int, len: *mut usize) -> *const c_char;

    /// Retrieves a const char pointer and updates an int pointer to the atom returned from the useratom callback the atom pointer is not NULL
    pub fn lua_tostringatom(state: *mut _LuaState, idx: c_int, atom: *mut c_int) -> *const c_char;

    /// Gets a const char poitner and an atom from the current namecall string if its not NULL
    pub fn lua_namecallatom(state: *mut _LuaState, atom: *mut c_int) -> *const c_char;

    /// Performs a luau length operator which retrieves the length of values (may invoke metamethods)
    pub fn lua_objlen(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Converts a value at the given index to a C function. That value must be a C function; otherwise, returns NULL.
    pub fn lua_tocfunction(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Converts a value at the given index to the lightuserdata's C pointer. That value must be a lightuserdata; otherwise, returns NULL.
    pub fn lua_tolightuserdata(state: *mut _LuaState, idx: c_int) -> *mut c_void;

    /// Converts a value at the given index to the lightuserdata's C pointer. That value must be a lightuserdata with the proper tag; otherwise, returns NULL.
    pub fn lua_tolightuserdatatagged(state: *mut _LuaState, idx: c_int, tag: Tag) -> *mut c_void;

    /// If the value at the given index is a full userdata, returns its memory-block address.
    /// If the value is a light userdata, returns its value (a pointer). Otherwise, returns NULL.
    pub fn lua_touserdata(state: *mut _LuaState, idx: c_int) -> *mut c_void;

    /// If the value at the given index is a full userdata and has a correct tag, returns its memory-block address.
    /// Otherwise, returns NULL.
    pub fn lua_touserdatatagged(state: *mut _LuaState, idx: c_int, tag: Tag) -> *mut c_void;

    /// Returns a full userdata's tag or -1 if there is no tag
    pub fn lua_userdatatag(state: *mut _LuaState, idx: c_int) -> Tag;

    /// Returns a light userdata's tag or -1 if there is no tag
    pub fn lua_lightuserdatatag(state: *mut _LuaState, idx: c_int) -> Tag;

    /// Converts the value at the given index to a Luau thread (represented as lua_State*).
    /// This value must be a thread; otherwise, the function returns NULL.
    pub fn lua_tothread(state: *mut _LuaState, idx: c_int) -> *mut _LuaState;

    /// Returns the buffer pointer and updates the value at the len pointer if its not NULL
    pub fn lua_tobuffer(state: *mut _LuaState, idx: c_int, len: *mut usize) -> *mut c_void;
    /// Converts the value at the given index to a generic C pointer (void*). The value can be a userdata, a table, a thread, a string, or a function; otherwise, lua_topointer returns NULL. Different objects will give different pointers. There is no way to convert the pointer back to its original value.
    ///
    /// Typically this function is used only for hashing and debug information.
    pub fn lua_topointer(state: *mut _LuaState, idx: c_int) -> *const c_void;
}

extern "C-unwind" {
    /// Pushes a nil value onto the stack
    pub fn lua_pushnil(state: *mut _LuaState);

    /// Pushes a double with value n onto the stack.
    pub fn lua_pushnumber(state: *mut _LuaState, n: LuaNumber);

    /// Pushes an integer with value n onto the stack.
    pub fn lua_pushinteger(state: *mut _LuaState, n: LuaInteger);

    /// Pushes an unsigned integer with value n onto the stack.
    pub fn lua_pushunsigned(state: *mut _LuaState, n: LuaUnsigned);

    #[cfg(feature = "luau_vector4")]
    /// Pushes a 4 value Luau vector onto the stack
    pub fn lua_pushvector(state: *mut _LuaState, x: c_float, y: c_float, z: c_float, w: c_float);

    #[cfg(not(feature = "luau_vector4"))]
    /// Pushes a 3 value Luau vector onto the stack
    pub fn lua_pushvector(state: *mut _LuaState, x: c_float, y: c_float, z: c_float);

    /// Pushes the string pointed to by s with size len onto the stack.
    /// Luau will make or reuse an internal copy of the given string, so the memory at s can be freed or reused immediately after the function returns.
    /// The string can contain any binary data, including embedded zeros.
    pub fn lua_pushlstring(state: *mut _LuaState, s: *const c_char, l: usize);

    /// Pushes the zero-terminated string pointed to by s onto the stack.
    /// Luau will make or reuse an internal copy of the given string, so the memory at s can be freed or reused immediately after the function returns.
    pub fn lua_pushstring(state: *mut _LuaState, s: *const c_char);

    // requires va_list, add when stable
    // fn lua_pushvfstring(state: *mut lua_State, fmt: *const c_char, argp: std::ffi::VaList) -> *const c_char;

    /// Pushes onto the stack a formatted string and returns a pointer to this string. This internally uses `vsnprintf` so all formatting applies.
    ///
    /// This function may raise errors due to memory overflow or an invalid conversion specifier.
    pub fn lua_pushfstringL(state: *mut _LuaState, fmt: *const c_char, ...) -> *const c_char;

    /// Pushes a new C closure onto the stack.
    ///
    /// When a C function is created, it is possible to associate some values with it, thus creating a C closure; these values are then accessible to the function whenever it is called.
    /// To associate values with a C function, first these values should be pushed onto the stack (when there are multiple values, the first value is pushed first).
    /// Then lua_pushcclosurek is called to create and push the C function onto the stack, with the argument n telling how many values should be associated with the function.
    /// lua_pushcclosurek also pops these values from the stack.
    ///
    /// The maximum value for n is 255.
    ///
    /// The continuation function is invoked when resumes acrossing it
    pub fn lua_pushcclosurek(
        state: *mut _LuaState,
        function: CFunction,
        debugname: *const c_char,
        nup: c_int,
        cont: Option<LuaContinuation>,
    );

    /// Pushes a boolean value with value b onto the stack.
    pub fn lua_pushboolean(state: *mut _LuaState, b: c_int);

    /// Pushes the thread represented by L onto the stack. Returns 1 if this thread is the main thread of its state.
    pub fn lua_pushthread(state: *mut _LuaState) -> c_int;
}

extern "C-unwind" {
    /// Pushes a new light userdata with a specified tag
    pub fn lua_pushlightuserdatatagged(state: *mut _LuaState, p: *mut c_void, tag: Tag);

    /// Creates a new sized userdata with a specified tag
    pub fn lua_newuserdatatagged(state: *mut _LuaState, sz: usize, tag: Tag) -> *mut c_void;

    /// Creates a new userdata object with a destructor callback which is invoked on GC
    pub fn lua_newuserdatadtor(state: *mut _LuaState, sz: usize, dtor: LuaDestructor);
}

extern "C-unwind" {
    /// Create a new luau buffer of a specified size
    ///
    /// Will error if it cannot be created
    pub fn lua_newbuffer(state: *mut _LuaState, sz: usize) -> *mut c_void;
}

extern "C-unwind" {
    /// Pushes onto the stack the value t[k], where t is the value at the given index and k is the value on the top of the stack.
    ///
    /// This function pops the key from the stack, pushing the resulting value in its place.
    /// As in Luau, this function may trigger a metamethod for the "index" event.
    ///
    /// Returns the type of the pushed value.
    pub fn lua_gettable(state: *mut _LuaState, idx: c_int) -> LuauType;

    /// Pushes onto the stack the value t[k], where t is the value at the given index.
    /// As in Luau, this function may trigger a metamethod for the "index" event.
    ///
    /// Returns the type of the pushed value.
    pub fn lua_getfield(state: *mut _LuaState, idx: c_int, k: *const c_char) -> LuauType;

    /// Pushes onto the stack the value t[k], where t is the value at the given index.
    /// This function will not trigger a metamethod for the "index" event.
    ///
    /// Returns the type of the pushed value.
    pub fn lua_rawgetfield(state: *mut _LuaState, idx: c_int, k: *const c_char) -> LuauType;

    /// Similar to lua_gettable, but does a raw access (i.e., without metamethods). The value at index must be a table.
    pub fn lua_rawget(state: *mut _LuaState, idx: c_int) -> LuauType;

    /// Pushes onto the stack the value t[n], where t is the table at the given index. The access is raw, that is, it does not use the __index metavalue.
    ///
    /// Returns the type of the pushed value.
    pub fn lua_rawgeti(state: *mut _LuaState, idx: c_int, idx: c_int) -> LuauType;

    /// Creates a table of a specified array sized and a specified size of the associative portion
    pub fn lua_createtable(state: *mut _LuaState, narr: c_int, nrec: c_int);
}

extern "C-unwind" {
    /// Sets a table at the index to be readonly or not with the enabled argument
    pub fn lua_setreadonly(state: *mut _LuaState, idx: c_int, enabled: c_int);

    /// Returns if a table at the index is readonly
    pub fn lua_getreadonly(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Sets the safeenv parameter which toggles specific optimizations, this should be called if environment mutation is done in a way which would affect code
    pub fn lua_setsafeenv(state: *mut _LuaState, idx: c_int, enabled: c_int);
}

extern "C-unwind" {
    /// If the value at the given index has a metatable, the function pushes that metatable onto the stack and returns 1.
    /// Otherwise, the function returns 0 and pushes nothing on the stack.
    pub fn lua_getmetatable(state: *mut _LuaState, objindex: c_int) -> c_int;

    /// Pushes onto the stack the environment table of the value at the given index.
    pub fn lua_getfenv(state: *mut _LuaState, idx: c_int);
}

extern "C-unwind" {
    /// Does the equivalent to t\[k\] = v, where t is the value at the given valid index, v is the value at the top of the stack, and k is the value just below the top.
    ///
    /// This function pops both the key and the value from the stack. As in Luau, this function may trigger a metamethod for the "newindex" event.
    pub fn lua_settable(state: *mut _LuaState, idx: c_int);

    /// Does the equivalent to t\[k\] = v, where t is the value at the given valid index and v is the value at the top of the stack.
    ///
    /// This function pops the value from the stack. As in Luau, this function may trigger a metamethod for the "newindex" event
    pub fn lua_setfield(state: *mut _LuaState, idx: c_int, k: *const c_char);

    /// Does the equivalent to t\[k\] = v, where t is the value at the given valid index and v is the value at the top of the stack.
    ///
    /// This function pops the value from the stack. This function will not trigger metamethods
    pub fn lua_rawsetfield(state: *mut _LuaState, idx: c_int, k: *const c_char);

    /// Similar to lua_settable, but does a raw assignment (i.e., without metamethods).
    pub fn lua_rawset(state: *mut _LuaState, idx: c_int);

    /// Does the equivalent of t[n] = v, where t is the value at the given valid index and v is the value at the top of the stack.
    ///
    /// This function pops the value from the stack. The assignment is raw; that is, it does not invoke metamethods.
    pub fn lua_rawseti(state: *mut _LuaState, idx: c_int, idx: c_int);

    /// Pops a table from the stack and sets it as the new metatable for the value at the given acceptable index.
    pub fn lua_setmetatable(state: *mut _LuaState, objindex: c_int) -> c_int;

    /// Pops a table from the stack and sets it as the new environment for the value at the given index.
    /// If the value at the given index is neither a function nor a thread nor a userdata, lua_setfenv returns 0. Otherwise it returns 1.
    pub fn lua_setfenv(state: *mut _LuaState, idx: c_int) -> c_int;
}

extern "C-unwind" {
    /// Loads a luau chunk into a function with a chunkname.
    ///
    /// Specifying an env will allow for optimizations to be performed, if you do not want to specify an env pass 0.
    pub fn luau_load(
        state: *mut _LuaState,
        chunkname: *const c_char,
        data: *const c_char,
        size: usize,
        env: c_int,
    ) -> c_int;

    /// Calls a function.
    ///
    /// To call a function you must use the following protocol: first, the function to be called is pushed onto the stack; then, the arguments to the function are pushed in direct order; that is, the first argument is pushed first. Finally you call lua_call; nargs is the number of arguments that you pushed onto the stack. All arguments and the function value are popped from the stack when the function is called. The function results are pushed onto the stack when the function returns. The number of results is adjusted to nresults, unless nresults is LUA_MULTRET. In this case, all results from the function are pushed. Luau takes care that the returned values fit into the stack space. The function results are pushed onto the stack in direct order (the first result is pushed first), so that after the call the last result is on the top of the stack.
    ///
    /// Any error inside the called function is propagated upwards with a Luau exception.
    pub fn lua_call(state: *mut _LuaState, nargs: c_int, nresults: c_int);

    /// Calls a function in protected mode.
    ///
    /// Both nargs and nresults have the same meaning as in lua_call. If there are no errors during the call, lua_pcall behaves exactly like lua_call. However, if there is any error, lua_pcall catches it, pushes a single value on the stack (the error message), and returns an error code. Like lua_call, lua_pcall always removes the function and its arguments from the stack.
    ///
    /// If errfunc is 0, then the error message returned on the stack is exactly the original error message. Otherwise, errfunc is the stack index of an error handler function. (In the current implementation, this index cannot be a pseudo-index.) In case of runtime errors, this function will be called with the error message and its return value will be the message returned on the stack by lua_pcall.
    ///
    /// Typically, the error handler function is used to add more debug information to the error message, such as a stack traceback. Such information cannot be gathered after the return of lua_pcall, since by then the stack has unwound.
    ///
    /// The lua_pcall function returns 0 in case of success or one of the following error codes:
    ///
    /// LUA_ERRRUN: a runtime error. \
    /// LUA_ERRMEM: memory allocation error. For such errors, Luau does not call the error handler function. \
    /// LUA_ERRERR: error while running the error handler function.
    pub fn lua_pcall(
        state: *mut _LuaState,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
    ) -> LuauStatus;
}

extern "C-unwind" {
    /// Yields a coroutine.
    ///
    /// This function should be called as the return expression of a C function, as follows:
    ///
    /// When a C function calls lua_yield in that way, the running coroutine suspends its execution, and the call to lua_resume that started this coroutine returns.
    /// The parameter nresults is the number of values from the stack that are passed as results to lua_resume.
    pub fn lua_yield(state: *mut _LuaState, nresults: c_int) -> c_int;

    /// Breaks execution of a coroutine
    ///
    /// This function could be called as the return expression of a C function
    ///
    /// When a C function calls lua_break in that way, the running coroutine stops its execution, and the call to lua_resume that started this coroutine returns.
    ///
    /// Will always return -1
    pub fn lua_break(state: *mut _LuaState) -> c_int;

    /// Starts and resumes a coroutine in the given thread L.
    ///
    /// To start a coroutine, you push the main function plus any arguments onto the empty stack of the thread. then you call lua_resume, with nargs being the number of arguments.
    /// This call returns when the coroutine suspends or finishes its execution. When it returns, *nresults is updated and the top of the stack contains the *nresults values passed to lua_yield or returned by the body function.
    /// lua_resume returns LUA_YIELD if the coroutine yields, LUA_OK if the coroutine finishes its execution without errors, or an error code in case of errors.
    /// In case of errors, the error object is on the top of the stack.
    ///
    /// To resume a coroutine, you remove the *nresults yielded values from its stack, push the values to be passed as results from yield, and then call lua_resume.
    ///
    /// The parameter from represents the coroutine that is resuming L. If there is no such coroutine, this parameter can be NULL.
    pub fn lua_resume(state: *mut _LuaState, from: *mut _LuaState, narg: c_int) -> LuauStatus;

    /// Resumes a coroutine in the thread L
    ///
    /// This errors the thread
    ///
    /// The parameter from represents the coroutine that is resuming L. If there is no such coroutine, this parameter can be NULL.
    pub fn lua_resumeerror(state: *mut _LuaState, from: *mut _LuaState) -> LuauStatus;

    /// Returns the execution status of the provided luau state
    pub fn lua_status(state: *mut _LuaState) -> LuauStatus;

    /// Returns true if the given coroutine can yield, and false otherwise.
    pub fn lua_isyieldable(state: *mut _LuaState) -> c_int;

    /// Returns an associated thread userdata pointer
    pub fn lua_getthreaddata(state: *mut _LuaState) -> *mut c_void;

    /// Sets a thread userdata pointer
    pub fn lua_setthreaddata(state: *mut _LuaState, data: *mut c_void);

    /// Returns the status of a coroutine
    pub fn lua_costatus(state: *mut _LuaState, co: *mut _LuaState) -> CoroutineStatus;
}

// Garbage collection options
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum GCOperation {
    /// Stop incremental garbage collection
    LUA_GCSTOP,

    /// Restart incremental garbage collection
    LUA_GCRESTART,

    /// run a full GC cycle; not recommended for latency sensitive applications
    LUA_GCCOLLECT,

    /// return the heap size in KB and the remainder in bytes
    LUA_GCCOUNT,

    /// return the heap size in KB and the remainder in bits
    LUA_GCCOUNTB,

    /// return 1 if GC is active (not stopped); note that GC may not be actively collecting even if it's running
    LUA_GCISRUNNING,

    /// Perform an explicit GC step, with the step size specified in KB.
    ///
    /// Garbage collection is handled by 'assists' that perform some amount of GC work matching pace of allocation
    /// explicit GC steps allow to perform some amount of work at custom points to offset the need for GC assists
    /// note that GC might also be paused for some duration (until bytes allocated meet the threshold)
    /// if an explicit step is performed during this pause, it will trigger the start of the next collection cycle
    LUA_GCSTEP,

    /// tune GC parameters G (goal), S (step multiplier) and step size (usually best left ignored)
    ///
    /// garbage collection is incremental and tries to maintain the heap size to balance memory and performance overhead
    /// this overhead is determined by G (goal) which is the ratio between total heap size and the amount of live data in it
    /// G is specified in percentages; by default G=200% which means that the heap is allowed to grow to ~2x the size of live data.
    ///
    /// collector tries to collect S% of allocated bytes by interrupting the application after step size bytes were allocated.
    /// when S is too small, collector may not be able to catch up and the effective goal that can be reached will be larger.
    /// S is specified in percentages; by default S=200% which means that collector will run at ~2x the pace of allocations.
    ///
    /// it is recommended to set S in the interval 100 / (G - 100), 100 + 100 / (G - 100)) with a minimum value of 150%; for example:
    /// - for G=200%, S should be in the interval 150% - 200%
    /// - for G=150%, S should be in the interval 200% - 300%
    /// - for G=125%, S should be in the interval 400% - 500%
    LUA_GCSETGOAL,

    /// Refer to LUA_GCSETGOAL
    LUA_GCSETSTEPMUL,

    /// Refer to LUA_GCSETGOAL
    LUA_GCSETSTEPSIZE,
}

// GC operation
extern "C-unwind" {
    /// Performs the GC operation specified by the what parameter
    pub fn lua_gc(state: *mut _LuaState, what: GCOperation, data: c_int) -> c_int;
}

// Memory statistics
extern "C-unwind" {
    /// Sets the active memory category of the provided state
    pub fn lua_setmemcat(state: *mut _LuaState, category: c_int);

    /// Gets the total allocation size of the provided category.
    ///
    /// Returns the total allocation size of all categories if the category provided is zero
    pub fn lua_totalbytes(state: *mut _LuaState, category: c_int) -> usize;
}

// Miscellaneous functions
extern "C-unwind" {
    /// Errors the current state
    ///
    /// Will not return
    ///
    /// Will cause an abort in the absence of a protected call
    pub fn lua_error(state: *mut _LuaState) -> !;

    /// Pops a key from the stack, and pushes a key-value pair from the table at the given index (the "next" pair after the given key).
    /// If there are no more elements in the table, then lua_next returns 0 (and pushes nothing).
    pub fn lua_next(state: *mut _LuaState, idx: c_int) -> c_int;

    /// Performs raw Luau iteration (array portion first, hash portion next) without invoking an iter metamethod
    pub fn lua_rawiter(state: *mut _LuaState, idx: c_int, iter: c_int) -> c_int;

    /// Concatenates the n values at the top of the stack, pops them, and leaves the result at the top.
    /// If n is 1, the result is the single value on the stack (that is, the function does nothing); if n is 0, the result is the empty string.
    ///
    /// Concatenation is performed following the usual semantics of Luau
    pub fn lua_concat(state: *mut _LuaState, idx: c_int);

    /// Performs pointer encryption
    pub fn lua_encodepointer(state: *mut _LuaState, p: usize) -> usize;

    /// Returns the value returned by the clock function in use by Luau
    pub fn lua_clock() -> c_double;

    /// Sets the tag of a full userdata
    pub fn lua_setuserdatatag(state: *mut _LuaState, idx: c_int, tag: Tag);

    /// Sets a full userdata tag destructor method
    ///
    /// This cannot be reassigned
    ///
    /// This destructor will be invoked for any of the tagged userdatas with that tag
    pub fn lua_setuserdatadtor(state: *mut _LuaState, tag: Tag, dtor: Option<LuaDestructor>);

    /// Gets a full userdata tag destructor method
    pub fn lua_getuserdatadtor(state: *mut _LuaState, tag: Tag) -> Option<LuaDestructor>;

    /// Sets a full userdata tag metatable
    ///
    /// This cannot be reassigned
    ///
    /// This metatable can be retrieved lua_getuserdatametatable
    pub fn lua_setuserdatametatable(state: *mut _LuaState, tag: Tag, idx: c_int);

    /// Get's a full userdata tag metatable
    pub fn lua_getuserdatametatable(state: *mut _LuaState, tag: Tag);

    /// Sets a lightuserdata tag name
    ///
    /// This cannot be reassigned
    ///
    /// This name can be retrieved lua_getlightuserdataname
    pub fn lua_setlightuserdataname(state: *mut _LuaState, tag: Tag, name: *const c_char);

    /// Gets a lightuserdata tag name
    pub fn lua_getlightuserdataname(state: *mut _LuaState, tag: Tag) -> *const c_char;

    /// Clones a function proto and its upvalues
    pub fn lua_clonefunction(state: *mut _LuaState, idx: c_int);

    /// Clears a table's while retaining its sizes
    pub fn lua_cleartable(state: *mut _LuaState, idx: c_int);

    /// Gets an allocation C function and sets the provided pointer to the pointer of the associated data
    pub fn lua_getallocf(state: *mut _LuaState, ud: *mut *mut c_void) -> LuaAlloc;
}

//
// Reference system, can be used to pin objects
//
pub const LUA_NOREF: c_int = -1;
pub const LUA_REFNIL: c_int = 0;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
/// Index into the Luau registry created by lua_ref
///
/// Reference indexes are not regular stack indexes and should be handled through lua_getref
pub struct RefIndex(pub(crate) c_int);

extern "C-unwind" {
    /// Creates a Luau reference which is used to "pin" a value in a specified place
    pub fn lua_ref(state: *mut _LuaState, idx: c_int) -> RefIndex;

    /// Removes a Luau value from its position, specified by the ref index, in the registry
    pub fn lua_unref(state: *mut _LuaState, r#ref: RefIndex);
}

/// Pushes the value for which a reference was made by `lua_ref`.
///
/// Returns the type of the value pushed
///
/// # Safety
/// You should uphold all safety invariants that apply to lua_rawgeti.
pub unsafe fn lua_getref(state: *mut _LuaState, ref_index: RefIndex) -> LuauType {
    lua_rawgeti(state, LUA_REGISTRYINDEX, ref_index.0)
}

//
// Debug API
//

// Maximum size for the description of the source of a function in debug information.
const LUA_IDSIZE: usize = 256;

/// Type for functions to be called on debug events.
pub type LuaHook = unsafe extern "C-unwind" fn(state: *mut _LuaState, ar: *mut LuaDebug);

pub type LuaCoverage = unsafe extern "C-unwind" fn(
    context: *mut c_void,
    function: *const c_char,
    linedefined: c_int,
    depth: c_int,
    hits: *const c_int,
    size: usize,
);

#[repr(C)]
pub struct LuaDebug {
    /// Field 'n'
    ///
    /// Name of the current function
    pub name: *const c_char, // (n)
    /// Set by field 's'
    ///
    /// Returns the "kind" of execution
    ///
    /// Values are:
    /// "Luau",
    /// "C",
    /// "main",
    /// "tail"
    pub what: *const c_char,

    /// Set by field 's'
    ///
    /// The source (chunkname) of an error.
    ///
    /// Tf the error is from a C function will be set to '=[C]'
    pub source: *const c_char,

    /// Set by field 's'
    ///
    /// Truncated form of the chunkname with some additional formatting rules not exceeding LUA_IDSIZE in length
    pub short_src: *const c_char,

    /// Set by field 's'
    ///
    /// The line on which the function was defined
    pub linedefined: c_int,

    /// Set by field 'l'
    ///
    /// Returns the current line in execution
    pub currentline: c_int,

    /// Set by field 'u'
    ///
    /// The number of upvalues for the function
    pub nupvals: c_uchar,

    /// Set by field 'a'
    ///
    /// The number of params accepted by the function
    pub nparams: c_uchar,

    /// Set by field 'a'
    ///
    /// 1 if the function is a vararg function
    ///
    /// 0 if it is not
    pub isvararg: c_char,

    /// Userdata field for lua_Debug
    ///
    /// Is only valid for luau_callhook
    pub userdata: *mut c_void, // only valid in luau_callhook

    /// Buffer used for short_src operation
    pub ssbuf: [c_char; LUA_IDSIZE],
}

extern "C-unwind" {
    /// Returns the state's current call stack depth
    pub fn lua_stackdepth(state: *mut _LuaState) -> c_int;

    /// Fills a LuaDebug structure with information specified by the `what` argument and pushes the function to the top of the stack.
    ///
    /// Level is expected to be a stack index (negative) or a callstack depth (positive)
    pub fn lua_getinfo(
        state: *mut _LuaState,
        level: c_int,
        what: *const c_char,
        ar: *mut LuaDebug,
    ) -> c_int;

    /// Returns the argument "n" passed at call level "level"
    pub fn lua_getargument(state: *mut _LuaState, level: c_int, idx: c_int) -> c_int;

    /// Gets the local `n` at call level `level` and pushes it to the top of the stack
    ///
    /// Returns the name of the local or NULL if it does not exist
    pub fn lua_getlocal(state: *mut _LuaState, level: c_int, idx: c_int) -> *const c_char;

    /// Sets the local `n` at call level `level` to a value popped from the top of the stack
    ///
    /// Returns the name of the local or NULL if it does not exist
    ///
    /// Note: This cannot be used for functions compiled with native codegen
    pub fn lua_setlocal(state: *mut _LuaState, level: c_int, idx: c_int) -> *const c_char;

    /// Gets upvalue `n` from function at `funcindex` and pushes it to the top of the stack
    ///
    /// Returns the name of the upvalue or NULL if the upvalue doesnt exist
    pub fn lua_getupvalue(state: *mut _LuaState, funcindex: c_int, idx: c_int) -> *const c_char;

    /// Retrieves and sets the upvalue `n` for function at `funcindex` to a value popped from the top of the stack
    ///
    /// Returns the name of the upvalue or NULL if the upvalue doesnt exist
    pub fn lua_setupvalue(state: *mut _LuaState, funcindex: c_int, idx: c_int) -> *const c_char;

    /// Sets single-step mode to be enabled or disabled, this allows for a callback to be invoked on every instruction and stops execution of native compiled code.
    pub fn lua_singlestep(state: *mut _LuaState, enabled: c_int);

    /// Sets a breakpoint status at the specified line and func index
    ///
    /// Will return the actual line that the breakpoint was placed on or -1 if it was unsuccessful
    pub fn lua_breakpoint(
        state: *mut _LuaState,
        funcindex: c_int,
        line: c_int,
        enabled: c_int,
    ) -> c_int;

    /// Invokes the LuaCoverage for every proto which contains coverage information
    ///
    /// Accepts a user provided context pointer
    pub fn lua_getcoverage(
        state: *mut _LuaState,
        funcindex: c_int,
        context: *mut c_void,
        callback: LuaCoverage,
    );

    /// Outputs a Luau stack trace with a maximum length of 4096
    ///
    /// The returned pointer is to a shared buffer used by all calls to this function so the returned value should be copied if it is to be retained past the break scope
    pub fn lua_debugtrace(state: *mut _LuaState) -> *const c_char;
}

#[repr(C)]
#[non_exhaustive]
/// Callbacks that can be used to reconfigure behavior of the VM dynamically.
///
/// These are shared between all coroutines.
pub struct LuaCallbacks {
    /// arbitrary userdata pointer that is never overwritten by Luau
    pub userdata: *mut c_void,

    /// gets called at safepoints (loop back edges, call/ret, gc) if set
    pub interrupt: Option<unsafe extern "C-unwind" fn(state: *mut _LuaState, gc: c_int)>,

    /// gets called when an unprotected error is raised (if longjmp is used)
    pub panic: Option<unsafe extern "C-unwind" fn(state: *mut _LuaState, errcode: LuauStatus)>,

    /// gets called when L is created (LP == parent) or destroyed (LP == NULL)
    pub userthread: Option<unsafe extern "C-unwind" fn(LP: *mut _LuaState, state: *mut _LuaState)>,

    /// gets called when a string is created; returned atom can be retrieved via tostringatom
    pub useratom: Option<unsafe extern "C-unwind" fn(s: *const c_char, l: usize) -> i16>,

    /// gets called when BREAK instruction is encountered
    pub debugbreak: Option<unsafe extern "C-unwind" fn(state: *mut _LuaState, ar: *mut LuaDebug)>,

    /// gets called after each instruction in single step mode
    pub debugstep: Option<unsafe extern "C-unwind" fn(state: *mut _LuaState, ar: *mut LuaDebug)>,

    /// gets called when thread execution is interrupted by break in another thread
    pub debuginterrupt:
        Option<unsafe extern "C-unwind" fn(state: *mut _LuaState, ar: *mut LuaDebug)>,

    /// gets called when protected call results in an error
    pub debugprotectederror: Option<unsafe extern "C-unwind" fn(state: *mut _LuaState)>,

    /// gets called when memory is allocated
    pub onallocate:
        Option<unsafe extern "C-unwind" fn(state: *mut _LuaState, osize: usize, nsize: usize)>,
}

extern "C" {
    /// Returns a pointer to the Luau callbacks struct
    pub fn lua_callbacks(state: *mut _LuaState) -> *mut LuaCallbacks;
}

/// Converts the Luau value at the given index to a Luau number. The Luau value must be a number or a string convertible to a number, otherwise returns 0.
pub unsafe fn lua_tonumber(state: *mut _LuaState, idx: c_int) -> LuaNumber {
    lua_tonumberx(state, idx, null_mut())
}

/// Converts the Luau value at the given index to the signed integral type. The Luau value must be an integer, or a number or string convertible to an integer; otherwise, returns 0.
pub unsafe fn lua_tointeger(state: *mut _LuaState, idx: c_int) -> LuaInteger {
    lua_tointegerx(state, idx, null_mut())
}

/// Converts the Luau value at the given index to the unsigned integral type. The Luau value must be an integer, or a number or string convertible to an integer; otherwise, returns 0.
pub unsafe fn lua_tounsigned(state: *mut _LuaState, idx: c_int) -> LuaUnsigned {
    lua_tounsignedx(state, idx, null_mut())
}

/// Pops n elements from the stack.
pub unsafe fn lua_pop(state: *mut _LuaState, idx: c_int) {
    lua_settop(state, -(idx) - 1);
}

pub unsafe fn lua_newtable(state: *mut _LuaState) {
    lua_createtable(state, 0, 0)
}
pub unsafe fn lua_newuserdata(state: *mut _LuaState, s: usize) -> *mut c_void {
    lua_newuserdatatagged(state, s, Tag(0))
}

pub unsafe fn lua_strlen(state: *mut _LuaState, i: c_int) -> c_int {
    lua_objlen(state, i)
}

/// Returns true if the value at `idx` is a function, false otherwise.
pub unsafe fn lua_isfunction(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TFUNCTION
}

/// Returns true if the value at `idx` is a table, false otherwise.
pub unsafe fn lua_istable(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TTABLE
}

/// Returns true if the value at `idx` is a light userdata, false otherwise.
pub unsafe fn lua_islightuserdata(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TLIGHTUSERDATA
}

/// Returns true if the value at `idx` is nil, false otherwise.
pub unsafe fn lua_isnil(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TNIL
}

/// Returns true if the value at `idx` is a boolean, false otherwise.
pub unsafe fn lua_isboolean(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TBOOLEAN
}

/// Returns true if the value at `idx` is a vector, false otherwise.
pub unsafe fn lua_isvector(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TVECTOR
}

/// Returns true if the value at `idx` is a thread, false otherwise.
pub unsafe fn lua_isthread(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TTHREAD
}

/// Returns true if the value at `idx` is a buffer, false otherwise.
pub unsafe fn lua_isbuffer(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TBUFFER
}

/// Returns true if the value at `n` doesn't exist, false otherwise.
pub unsafe fn lua_isnone(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) == LuauType::LUA_TNONE
}

/// Returns true if the value at `n` doesnt exist or is nil
pub unsafe fn lua_isnoneornil(state: *mut _LuaState, idx: c_int) -> bool {
    lua_type(state, idx) <= LuauType::LUA_TNIL
}

/// Pushes a static string to Luau
pub unsafe fn lua_pushliteral(state: *mut _LuaState, s: &'static str) {
    lua_pushlstring(state, s.as_ptr() as _, s.len())
}

/// Pushes a C function with no upvalues and the given debug name
pub unsafe fn lua_pushcfunction(state: *mut _LuaState, func: CFunction, debugname: *const c_char) {
    lua_pushcclosurek(state, func, debugname, 0, None);
}

/// Pushes a C function with the given amount of upvalues
pub unsafe fn lua_pushcclosure(
    state: *mut _LuaState,
    func: CFunction,
    debugname: *const c_char,
    nup: c_int,
) {
    lua_pushcclosurek(state, func, debugname, nup, None)
}

pub unsafe fn lua_pushlightuserdata(state: *mut _LuaState, p: *mut c_void) {
    lua_pushlightuserdatatagged(state, p, Tag(0))
}

/// Pops a value from the stack and sets it as the new value of global name.
pub unsafe fn lua_setglobal(state: *mut _LuaState, s: *const c_char) {
    lua_setfield(state, LUA_GLOBALSINDEX, s)
}

/// Pushes onto the stack the value of the global name. Returns the type of that value.
pub unsafe fn lua_getglobal(state: *mut _LuaState, s: *const c_char) -> LuauType {
    lua_getfield(state, LUA_GLOBALSINDEX, s)
}

pub unsafe fn lua_tostring(state: *mut _LuaState, i: c_int) -> *const c_char {
    lua_tolstring(state, i, null_mut())
}

#[macro_export]
macro_rules! lua_pushformat {
    ($state:expr, $fmt:expr, $($args:tt)*) => {
        let string = std::fmt::format(format_args!($fmt, $($args)*));
        $crate::ffi::prelude::lua_pushlstring($state, string.as_str().as_ptr() as _, string.len())
    };
}
