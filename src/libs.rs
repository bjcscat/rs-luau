use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

#[derive(Debug, Clone, Copy)]
pub struct LuauLibs(u32);

impl LuauLibs {
    /// Loads all luau libs
    pub const ALL_LIBS: LuauLibs = LuauLibs(u32::MAX);

    /// The base library (pcall, unpack, print, etc)
    pub const LIB_BASE: LuauLibs = LuauLibs(1);
    /// The `coroutine` library
    pub const LIB_COROUTINE: LuauLibs = LuauLibs(1 << 1);
    /// The `table` library
    pub const LIB_TABLE: LuauLibs = LuauLibs(1 << 2);
    /// The `os` library
    pub const LIB_OS: LuauLibs = LuauLibs(1 << 3);
    /// The `string` library
    pub const LIB_STRING: LuauLibs = LuauLibs(1 << 4);
    /// The `math` library
    pub const LIB_MATH: LuauLibs = LuauLibs(1 << 5);
    /// The `debug` library
    pub const LIB_DEBUG: LuauLibs = LuauLibs(1 << 6);
    /// The `utf8` library
    pub const LIB_UTF8: LuauLibs = LuauLibs(1 << 7);
    /// The `bit32` library
    pub const LIB_BIT32: LuauLibs = LuauLibs(1 << 8);
    /// The `buffer` library
    pub const LIB_BUFFER: LuauLibs = LuauLibs(1 << 9);
    /// The `vector` library
    pub const LIB_VECTOR: LuauLibs = LuauLibs(1 << 10);

    pub fn has(&self, lib: LuauLibs) -> bool {
        self.0 & lib.0 == self.0
    }
}

impl BitXor for LuauLibs {
    type Output = LuauLibs;

    fn bitxor(self, rhs: Self) -> Self::Output {
        LuauLibs(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for LuauLibs {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = LuauLibs(self.0 ^ rhs.0)
    }
}

impl BitOr for LuauLibs {
    type Output = LuauLibs;

    fn bitor(self, rhs: Self) -> Self::Output {
        LuauLibs(self.0 | rhs.0)
    }
}

impl BitOrAssign for LuauLibs {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0)
    }
}

impl BitAnd for LuauLibs {
    type Output = LuauLibs;

    fn bitand(self, rhs: Self) -> Self::Output {
        LuauLibs(self.0 & rhs.0)
    }
}

impl BitAndAssign for LuauLibs {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = LuauLibs(self.0 & rhs.0)
    }
}
