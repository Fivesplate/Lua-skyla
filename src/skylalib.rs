// skylalib.rs - Skyla/Lua standard library registration (Rust translation of lualib.h)
// This module defines library names, keys, and open functions for all standard libraries.

use crate::lstate::LuaState;

// Version suffix for environment variable names
pub const LUA_VERSUFFIX: &str = "_5_4"; // Adjust as needed

// Library names
pub const LUA_LOADLIBNAME: &str = "package";
pub const LUA_COLIBNAME: &str = "coroutine";
pub const LUA_DBLIBNAME: &str = "debug";
pub const LUA_IOLIBNAME: &str = "io";
pub const LUA_MATHLIBNAME: &str = "math";
pub const LUA_OSLIBNAME: &str = "os";
pub const LUA_STRLIBNAME: &str = "string";
pub const LUA_TABLIBNAME: &str = "table";
pub const LUA_UTF8LIBNAME: &str = "utf8";

// Library open functions (to be implemented in their respective modules)
pub fn open_base(state: &mut LuaState) { /* ... */ }
pub fn open_package(state: &mut LuaState) { /* ... */ }
pub fn open_coroutine(state: &mut LuaState) { /* ... */ }
pub fn open_debug(state: &mut LuaState) { /* ... */ }
pub fn open_io(state: &mut LuaState) { /* ... */ }
pub fn open_math(state: &mut LuaState) { /* ... */ }
pub fn open_os(state: &mut LuaState) { /* ... */ }
pub fn open_string(state: &mut LuaState) { /* ... */ }
pub fn open_table(state: &mut LuaState) { /* ... */ }
pub fn open_utf8(state: &mut LuaState) { /* ... */ }

/// Open all standard libraries (call this from your VM entry point)
pub fn open_libs(state: &mut LuaState) {
    open_base(state);
    open_package(state);
    open_coroutine(state);
    open_debug(state);
    open_io(state);
    open_math(state);
    open_os(state);
    open_string(state);
    open_table(state);
    open_utf8(state);
}

