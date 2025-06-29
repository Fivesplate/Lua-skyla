//! linit.rs - Lua state and library initialization (inspired by Lua's linit.c)

use crate::lstate::lua_State;
use crate::lbaselib::luaopen_base;
use crate::ltablib::luaopen_table;
use crate::lstrlib::luaopen_string;
use crate::lmathlib::luaopen_math;
use crate::ldblib::luaopen_debug;
use crate::loslib::luaopen_os;
use crate::lcorolib::luaopen_coroutine;
use crate::liolib::luaopen_io;
use crate::lutf8lib::luaopen_utf8;
// Add more library modules as needed

/// List of standard libraries to open
const LUA_LIBS: &[(&str, fn(*mut lua_State) -> i32)] = &[
    ("_G", luaopen_base),
    ("table", luaopen_table),
    ("string", luaopen_string),
    ("math", luaopen_math),
    ("debug", luaopen_debug),
    ("os", luaopen_os),
    ("coroutine", luaopen_coroutine),
    ("io", luaopen_io),
    ("utf8", luaopen_utf8),
    // Add more libraries here
];

/// Example: Library metadata structure for documentation and introspection
#[derive(Debug, Clone)]
pub struct LuaLibInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
}

/// Example: Metadata for each standard library
const LUA_LIBS_INFO: &[LuaLibInfo] = &[
    LuaLibInfo { name: "_G", description: "Base functions", version: "1.0" },
    LuaLibInfo { name: "table", description: "Table manipulation", version: "1.0" },
    LuaLibInfo { name: "string", description: "String manipulation", version: "1.0" },
    LuaLibInfo { name: "math", description: "Mathematical functions", version: "1.0" },
    LuaLibInfo { name: "debug", description: "Debug facilities", version: "1.0" },
    LuaLibInfo { name: "os", description: "Operating system facilities", version: "1.0" },
    LuaLibInfo { name: "coroutine", description: "Coroutine manipulation", version: "1.0" },
    LuaLibInfo { name: "io", description: "Input and output facilities", version: "1.0" },
    LuaLibInfo { name: "utf8", description: "UTF-8 support", version: "1.0" },
    // Add more metadata here
];

/// Open a single library by name
pub unsafe fn luaL_openlib_by_name(L: *mut lua_State, libname: &str) {
    for &(name, openf) in LUA_LIBS {
        if name == libname {
            crate::lapi::luaL_requiref(L, name, Some(openf), 1);
            crate::lapi::lua_pop(L, 1);
            break;
        }
    }
}

/// Open all standard libraries
pub unsafe fn luaL_openlibs(L: *mut lua_State) {
    for &(name, openf) in LUA_LIBS {
        crate::lapi::luaL_requiref(L, name, Some(openf), 1);
        crate::lapi::lua_pop(L, 1);
    }
}

/// Optionally, allow registering custom libraries at runtime
pub fn luaL_register_custom_lib(libname: &'static str, openf: fn(*mut lua_State) -> i32) {
    // In a real implementation, you might push to a global registry or extend LUA_LIBS
    // This is a placeholder for extensibility
    // e.g., LUA_LIBS.push((libname, openf));
}

/// Helper: Check if a library is available by name
pub fn luaL_has_lib(libname: &str) -> bool {
    LUA_LIBS.iter().any(|(name, _)| *name == libname)
}

/// Helper: List all available standard libraries
pub fn luaL_list_libs() -> Vec<&'static str> {
    LUA_LIBS.iter().map(|(name, _)| *name).collect()
}

/// Print detailed info about all libraries
pub fn luaL_print_libs_info() {
    println!("Lua Standard Libraries:");
    for info in LUA_LIBS_INFO {
        println!("- {} (v{}): {}", info.name, info.version, info.description);
    }
}

/// Example: Get library info by name
pub fn luaL_get_lib_info(libname: &str) -> Option<&'static LuaLibInfo> {
    LUA_LIBS_INFO.iter().find(|info| info.name == libname)
}

/// Example: Print info for a specific library
pub fn luaL_print_lib_info(libname: &str) {
    if let Some(info) = luaL_get_lib_info(libname) {
        println!("Library: {}\nVersion: {}\nDescription: {}", info.name, info.version, info.description);
    } else {
        println!("Library '{}' not found.", libname);
    }
}

/// Example: Open a subset of libraries (by names)
pub unsafe fn luaL_openlibs_subset(L: *mut lua_State, libs: &[&str]) {
    for &libname in libs {
        for &(name, openf) in LUA_LIBS {
            if name == libname {
                crate::lapi::luaL_requiref(L, name, Some(openf), 1);
                crate::lapi::lua_pop(L, 1);
                break;
            }
        }
    }
}

/// Example: Custom initialization hook (user can override)
pub fn luaL_custom_init(_L: *mut lua_State) {
    // User can override this function to register custom libraries or perform extra setup
}

/// Example: Unit tests for linit.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_has_lib() {
        assert!(luaL_has_lib("math"));
        assert!(!luaL_has_lib("nonexistent"));
    }

    #[test]
    fn test_list_libs() {
        let libs = luaL_list_libs();
        assert!(libs.contains(&"math"));
        assert!(libs.contains(&"string"));
    }

    #[test]
    fn test_openlibs_subset() {
        // This is a placeholder; in a real test, you would use a mock lua_State
        unsafe {
            luaL_openlibs_subset(ptr::null_mut(), &["math", "string"]);
        }
    }

    #[test]
    fn test_print_libs() {
        luaL_print_libs();
    }
}

/// Documentation for users
///
/// Call `luaL_openlibs(L)` after creating a new Lua state to load all standard libraries.
/// Use `luaL_openlib_by_name(L, "math")` to load a single library.
///
/// Extended documentation for users
///
/// - Use `luaL_print_libs_info()` to print all library metadata.
/// - Use `luaL_get_lib_info(libname)` to get info for a specific library.
/// - Use `luaL_print_lib_info(libname)` to print info for a specific library.
///
/// Documentation for extending linit.rs
///
/// - To add a new standard library, add it to LUA_LIBS and import its open function.
/// - To register a custom library at runtime, use luaL_register_custom_lib.
/// - To open only a subset of libraries, use luaL_openlibs_subset.
/// - To check for a library, use luaL_has_lib.
/// - To list all libraries, use luaL_list_libs.
