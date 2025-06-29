//! llimits.rs - Lua limits and compile-time constants (Rust translation of llimits.h)

// Integer and floating-point types
pub type LuaInt = i32;
pub type LuaNum = f64;

// Maximum values for Lua integers and numbers
pub const LUA_MAXINTEGER: LuaInt = std::i32::MAX;
pub const LUA_MININTEGER: LuaInt = std::i32::MIN;
pub const LUA_MAXNUMBER: LuaNum = std::f64::MAX;
pub const LUA_MINNUMBER: LuaNum = std::f64::MIN;

// Stack and call limits
pub const LUAI_MAXSTACK: usize = 1000000;
pub const LUAI_MAXCALLS: usize = 20000;
pub const LUAI_MAXCCALLS: usize = 200;
pub const LUAI_MAXUPVAL: usize = 255;

// String and table limits
pub const LUAI_MAXSHORTLEN: usize = 40;
pub const MAX_SIZET: usize = std::usize::MAX;
pub const MAX_SIZE: usize = std::i32::MAX as usize;

// Buffer and memory limits
pub const LUAL_BUFFERSIZE: usize = 8192;
pub const LUAI_MAXBUFFER: usize = 1024 * 1024;

// Other limits
pub const LUAI_MAXALIGN: usize = 8;
pub const LUAI_MAXCLOSUREVARS: usize = 200;

// Useful macros as Rust const fns
#[inline(always)]
pub const fn lua_imin(a: LuaInt, b: LuaInt) -> LuaInt { if a < b { a } else { b } }
#[inline(always)]
pub const fn lua_imax(a: LuaInt, b: LuaInt) -> LuaInt { if a > b { a } else { b } }

// Alignment helpers
#[inline(always)]
pub const fn lua_align(n: usize, align: usize) -> usize {
    (n + (align - 1)) & !(align - 1)
}

// Maximum size for short strings (Lua 5.3+)
pub const LUAI_MAXSHORTSTR: usize = 40;

// Maximum length for identifiers
pub const LUAI_MAXIDENT: usize = 256;

// Maximum number of local variables per function
pub const LUAI_MAXVARS: usize = 200;

// Maximum number of parameters per function
pub const LUAI_MAXPARAMS: usize = 200;

// Maximum number of instructions per function
pub const LUAI_MAXINSTR: usize = 65536;

// Maximum number of constants per function
pub const LUAI_MAXCONSTS: usize = 65536;

// Maximum number of prototypes per function
pub const LUAI_MAXPROTOS: usize = 65536;

// Maximum number of upvalues per closure
pub const LUAI_MAXUPVALUES: usize = 60;

// Maximum number of lines per function
pub const LUAI_MAXLINES: usize = 1000000;

// Maximum number of fields in a table
pub const LUAI_MAXFIELDS: usize = 1000000;

// Maximum number of open upvalues
pub const LUAI_MAXOPENUPVAL: usize = 1000;

// Maximum number of threads
pub const LUAI_MAXTHREADS: usize = 1000;

// Maximum number of metatables
pub const LUAI_MAXMETATABLES: usize = 256;

// Maximum number of GC objects
pub const LUAI_MAXGCOBJECTS: usize = 1000000;

// Maximum recursion depth for parser
pub const LUAI_MAXPARSERDEPTH: usize = 200;

// Maximum recursion depth for VM
pub const LUAI_MAXVMDEPTH: usize = 200;

// Maximum number of error messages
pub const LUAI_MAXERRORS: usize = 100;

// Compile-time assertions (Rust style)
const _: () = assert!(LUAI_MAXUPVAL <= 255);
const _: () = assert!(LUAI_MAXSTACK > 150);

// Platform-specific (dummy for now)
#[cfg(target_pointer_width = "64")]
pub const LUA_PTR_SIZE: usize = 8;
#[cfg(target_pointer_width = "32")]
pub const LUA_PTR_SIZE: usize = 4;

// Type aliases for compatibility
pub type lu_byte = u8;
pub type l_mem = isize;
pub type lu_mem = usize;

// Utility: number of bits in a type
#[inline(always)]
pub const fn l_numbits<T>() -> usize { std::mem::size_of::<T>() * 8 }

// MAX_LMEM: maximum value for l_mem
pub const MAX_LMEM: l_mem = ((1 as lu_mem) << (l_numbits::<l_mem>() - 1)) - 1;

// Small natural types
pub type ls_byte = i8;

// Type for thread status/error codes
pub type TStatus = lu_byte;

// APIstatus: convert TStatus to int
#[inline(always)]
pub const fn APIstatus(st: TStatus) -> i32 { st as i32 }

// log2maxs: floor of log2 of max signed value for type T
#[inline(always)]
pub const fn log2maxs<T>() -> usize { l_numbits::<T>() - 2 }

// ispow2: test if unsigned value is a power of 2 (or zero)
#[inline(always)]
pub const fn ispow2(x: usize) -> bool { (x & (x - 1)) == 0 }

// LL: number of chars in a literal string without the ending \0
#[macro_export]
macro_rules! LL {
    ($x:expr) => { $x.len() };
}

// point2uint: convert pointer to unsigned integer (for hashing only)
#[inline(always)]
pub fn point2uint<T>(p: *const T) -> u32 {
    (p as usize & u32::MAX as usize) as u32
}

// Internal assertion macros (Rust style)
#[macro_export]
macro_rules! lua_assert {
    ($cond:expr) => { debug_assert!($cond) };
}

#[macro_export]
macro_rules! check_exp {
    ($cond:expr, $e:expr) => {{ lua_assert!($cond); $e }};
}

#[macro_export]
macro_rules! lua_longassert {
    ($cond:expr) => { lua_assert!($cond) };
}

// UNUSED macro
#[macro_export]
macro_rules! UNUSED {
    ($x:expr) => { let _ = &$x; };
}

// Type casts (Rust style)
#[inline(always)]
pub const fn cast<T, U>(x: U) -> T where U: Into<T> { x.into() }

// Numeric operations (Rust style)
#[inline(always)]
pub fn luai_numadd(a: LuaNum, b: LuaNum) -> LuaNum { a + b }
#[inline(always)]
pub fn luai_numsub(a: LuaNum, b: LuaNum) -> LuaNum { a - b }
#[inline(always)]
pub fn luai_nummul(a: LuaNum, b: LuaNum) -> LuaNum { a * b }
#[inline(always)]
pub fn luai_numdiv(a: LuaNum, b: LuaNum) -> LuaNum { a / b }
#[inline(always)]
pub fn luai_numunm(a: LuaNum) -> LuaNum { -a }
#[inline(always)]
pub fn luai_numeq(a: LuaNum, b: LuaNum) -> bool { a == b }
#[inline(always)]
pub fn luai_numlt(a: LuaNum, b: LuaNum) -> bool { a < b }
#[inline(always)]
pub fn luai_numle(a: LuaNum, b: LuaNum) -> bool { a <= b }
#[inline(always)]
pub fn luai_numgt(a: LuaNum, b: LuaNum) -> bool { a > b }
#[inline(always)]
pub fn luai_numge(a: LuaNum, b: LuaNum) -> bool { a >= b }
#[inline(always)]
pub fn luai_numisnan(a: LuaNum) -> bool { a != a }
