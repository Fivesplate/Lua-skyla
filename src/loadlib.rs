//! loadlib.rs - Dynamic library loader and package system for Lua VM (Rust port)
// Inspired by Lua's loadlib.c, using Rust's libloading and std abstractions

mod lualib;
mod llimits;
mod lauxlib;
mod lua;

use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use libloading::{Library, Symbol};

use crate::lualib::*;
use crate::llimits::*;
use crate::lauxlib::*;
use crate::lua::*;

/// Prefix for open functions in C libraries
const LUA_POF: &str = "luaopen_";
/// Separator for open functions in C libraries
const LUA_OFSEP: &str = "_";

/// Key for table in the registry that keeps handles for all loaded C libraries
const CLIBS: &str = "_CLIBS";

/// Error codes for lookforfunc
const ERRLIB: i32 = 1;
const ERRFUNC: i32 = 2;

/// Global registry of loaded libraries (path -> Library)
lazy_static::lazy_static! {
    static ref LIB_REGISTRY: Mutex<HashMap<String, Library>> = Mutex::new(HashMap::new());
}

/// Load a dynamic library and return a handle
fn load_library(path: &str) -> Result<Library, String> {
    Library::new(path).map_err(|e| e.to_string())
}

/// Find a symbol in a loaded library
unsafe fn find_symbol<T>(lib: &Library, sym: &str) -> Result<Symbol<T>, String> {
    let cstr = CString::new(sym).unwrap();
    lib.get::<T>(cstr.as_bytes_with_nul()).map_err(|e| e.to_string())
}

/// Look for a C function named 'sym' in a dynamically loaded library 'path'.
/// Returns Ok(Some(fn_ptr)) if found, Ok(None) if only loading the library, Err if error.
fn lookforfunc(path: &str, sym: &str) -> Result<Option<*const ()>, (i32, String)> {
    let mut reg = LIB_REGISTRY.lock().unwrap();
    let lib = if let Some(lib) = reg.get(path) {
        lib
    } else {
        match load_library(path) {
            Ok(lib) => {
                reg.insert(path.to_string(), lib);
                reg.get(path).unwrap()
            },
            Err(e) => return Err((ERRLIB, e)),
        }
    };
    if sym == "*" {
        return Ok(None);
    }
    unsafe {
        match find_symbol::<unsafe extern "C" fn()>(lib, sym) {
            Ok(symbol) => Ok(Some(*symbol as *const ())),
            Err(e) => Err((ERRFUNC, e)),
        }
    }
}

/// Search path logic (simplified)
pub fn search_path(name: &str, path: &str, sep: &str, dirsep: &str) -> Result<String, String> {
    let mut tried = Vec::new();
    let mut found = None;
    for template in path.split(';') {
        let candidate = template.replace("?", name).replace(sep, dirsep);
        if std::fs::metadata(&candidate).is_ok() {
            found = Some(candidate);
            break;
        } else {
            tried.push(candidate);
        }
    }
    found.ok_or_else(|| format!("no file found in paths: {:?}", tried))
}

/// Package table and require logic (skeleton)
pub struct Package {
    pub loaded: HashMap<String, bool>,
    pub preload: HashMap<String, fn()>,
    pub cpath: String,
    pub path: String,
}

impl Package {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
            preload: HashMap::new(),
            cpath: String::from("./?.so;./lib?.so"),
            path: String::from("./?.lua;./?/init.lua"),
        }
    }

    /// Simulate 'require' for a module
    pub fn require(&mut self, name: &str) -> Result<(), String> {
        if self.loaded.get(name).copied().unwrap_or(false) {
            return Ok(());
        }
        // Try preload first
        if let Some(init) = self.preload.get(name) {
            init();
            self.loaded.insert(name.to_string(), true);
            return Ok(());
        }
        // Try C library
        let cpath = self.cpath.clone();
        let filename = search_path(name, &cpath, ".", std::path::MAIN_SEPARATOR_STR)?;
        let sym = format!("{}{}", LUA_POF, name.replace('.', LUA_OFSEP));
        match lookforfunc(&filename, &sym) {
            Ok(Some(_fn_ptr)) => {
                // TODO: Actually call/init the function pointer
                self.loaded.insert(name.to_string(), true);
                Ok(())
            },
            Ok(None) => Err("Library loaded but no function found".to_string()),
            Err((_errcode, msg)) => Err(msg),
        }
    }
}

/// Add support for Lua file loading, error reporting, and searchers
/// Error type for package operations
#[derive(Debug)]
pub enum PackageError {
    NotFound(String),
    LoadError(String),
    SymbolError(String),
    IoError(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for PackageError {
    fn from(e: std::io::Error) -> Self {
        PackageError::IoError(e)
    }
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageError::NotFound(s) => write!(f, "Not found: {}", s),
            PackageError::LoadError(s) => write!(f, "Load error: {}", s),
            PackageError::SymbolError(s) => write!(f, "Symbol error: {}", s),
            PackageError::IoError(e) => write!(f, "IO error: {}", e),
            PackageError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

/// Searcher trait for extensible searchers
pub trait Searcher {
    fn search(&self, pkg: &mut Package, name: &str) -> Result<(), PackageError>;
}

/// Lua file searcher
pub struct LuaFileSearcher;
impl Searcher for LuaFileSearcher {
    fn search(&self, pkg: &mut Package, name: &str) -> Result<(), PackageError> {
        let filename = search_path(name, &pkg.path, ".", std::path::MAIN_SEPARATOR_STR)
            .map_err(PackageError::NotFound)?;
        // Simulate loading and running the Lua file
        let mut file = fs::File::open(&filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        // TODO: Actually parse/execute Lua code
        println!("[LuaFileSearcher] Loaded Lua file: {}", filename);
        pkg.loaded.insert(name.to_string(), true);
        Ok(())
    }
}

/// C library searcher
pub struct CLibrarySearcher;
impl Searcher for CLibrarySearcher {
    fn search(&self, pkg: &mut Package, name: &str) -> Result<(), PackageError> {
        let cpath = pkg.cpath.clone();
        let filename = search_path(name, &cpath, ".", std::path::MAIN_SEPARATOR_STR)
            .map_err(PackageError::NotFound)?;
        let sym = format!("{}{}", LUA_POF, name.replace('.', LUA_OFSEP));
        match lookforfunc(&filename, &sym) {
            Ok(Some(_fn_ptr)) => {
                // TODO: Actually call/init the function pointer
                println!("[CLibrarySearcher] Loaded C library: {} symbol: {}", filename, sym);
                pkg.loaded.insert(name.to_string(), true);
                Ok(())
            },
            Ok(None) => Err(PackageError::SymbolError("Library loaded but no function found".to_string())),
            Err((_errcode, msg)) => Err(PackageError::LoadError(msg)),
        }
    }
}

/// Preload searcher
pub struct PreloadSearcher;
impl Searcher for PreloadSearcher {
    fn search(&self, pkg: &mut Package, name: &str) -> Result<(), PackageError> {
        if let Some(init) = pkg.preload.get(name) {
            init();
            pkg.loaded.insert(name.to_string(), true);
            println!("[PreloadSearcher] Loaded from preload: {}", name);
            Ok(())
        } else {
            Err(PackageError::NotFound(format!("No preload for {}", name)))
        }
    }
}

/// Package with searchers
pub struct PackageExt {
    pub pkg: Package,
    pub searchers: Vec<Box<dyn Searcher + Send + Sync>>,
}

impl PackageExt {
    pub fn new() -> Self {
        Self {
            pkg: Package::new(),
            searchers: vec![
                Box::new(PreloadSearcher),
                Box::new(LuaFileSearcher),
                Box::new(CLibrarySearcher),
            ],
        }
    }

    /// Simulate 'require' with searchers
    pub fn require(&mut self, name: &str) -> Result<(), PackageError> {
        if self.pkg.loaded.get(name).copied().unwrap_or(false) {
            return Ok(());
        }
        for searcher in &self.searchers {
            match searcher.search(&mut self.pkg, name) {
                Ok(_) => return Ok(()),
                Err(PackageError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(PackageError::NotFound(format!("Module '{}' not found", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_search_path() {
        let path = "./?.so;./lib?.so";
        let name = "testmod";
        let result = search_path(name, path, ".", "/");
        assert!(result.is_err() || result.as_ref().unwrap().contains("testmod"));
    }
    #[test]
    fn test_package_require() {
        let mut pkg = Package::new();
        // Simulate preload
        pkg.preload.insert("foo".to_string(), || println!("init foo"));
        assert!(pkg.require("foo").is_ok());
        assert!(pkg.loaded["foo"]);
    }
}

#[cfg(test)]
mod ext_tests {
    use super::*;
    #[test]
    fn test_package_ext_preload() {
        let mut pkg = PackageExt::new();
        pkg.pkg.preload.insert("bar".to_string(), || println!("init bar"));
        assert!(pkg.require("bar").is_ok());
        assert!(pkg.pkg.loaded["bar"]);
    }
    #[test]
    fn test_package_ext_notfound() {
        let mut pkg = PackageExt::new();
        let result = pkg.require("notfound");
        assert!(matches!(result, Err(PackageError::NotFound(_))));
    }
}
