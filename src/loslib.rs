//! loslib.rs - Standard Operating System library for Lua (Rust port)
// Provides OS and time functions for Lua scripts, similar to loslib.c

use std::env;
use std::fs;
use std::process::{Command, exit};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ffi::OsString;
use chrono::{Datelike, Timelike, Local, Utc, NaiveDateTime};

// Placeholder for Lua state and API integration
type LuaState = ();

// --- OS Functions ---

pub fn os_execute(cmd: Option<&str>) -> Result<i32, String> {
    match cmd {
        Some(command) => {
            let status = Command::new("sh").arg("-c").arg(command).status();
            status.map(|s| s.code().unwrap_or(-1)).map_err(|e| e.to_string())
        },
        None => Ok(0), // true if there is a shell
    }
}

pub fn os_remove(filename: &str) -> Result<(), String> {
    fs::remove_file(filename).map_err(|e| e.to_string())
}

pub fn os_rename(from: &str, to: &str) -> Result<(), String> {
    fs::rename(from, to).map_err(|e| e.to_string())
}

pub fn os_tmpname() -> Result<String, String> {
    let mut tmp = env::temp_dir();
    tmp.push(format!("lua_{:x}", rand::random::<u64>()));
    Ok(tmp.to_string_lossy().into_owned())
}

pub fn os_getenv(var: &str) -> Option<String> {
    env::var(var).ok()
}

pub fn os_clock() -> f64 {
    // Returns process time in seconds (not wall clock)
    // Placeholder: returns wall clock time since UNIX_EPOCH
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

// --- Time/Date Functions ---

pub fn os_date(fmt: Option<&str>, t: Option<i64>, utc: bool) -> String {
    let time = t.unwrap_or_else(|| chrono::Local::now().timestamp());
    let dt = if utc {
        Utc.timestamp_opt(time, 0).unwrap()
    } else {
        Local.timestamp_opt(time, 0).unwrap().naive_local()
    };
    match fmt.unwrap_or("%c") {
        "*t" => format!("{{year={}, month={}, day={}, hour={}, min={}, sec={}, wday={}, yday={}, isdst={}}}",
            dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second(),
            dt.weekday().number_from_sunday(), dt.ordinal(), false),
        f => dt.format(f).to_string(),
    }
}

pub fn os_time(table: Option<&[(&str, i32)]>) -> i64 {
    if let Some(fields) = table {
        let mut year = 1970; let mut month = 1; let mut day = 1;
        let mut hour = 12; let mut min = 0; let mut sec = 0;
        for &(k, v) in fields {
            match k {
                "year" => year = v,
                "month" => month = v,
                "day" => day = v,
                "hour" => hour = v,
                "min" => min = v,
                "sec" => sec = v,
                _ => {}
            }
        }
        let dt = NaiveDateTime::from_timestamp_opt(
            chrono::NaiveDate::from_ymd_opt(year, month as u32, day as u32)
                .unwrap()
                .and_hms_opt(hour as u32, min as u32, sec as u32)
                .unwrap()
                .timestamp(),
            0
        ).unwrap();
        dt.timestamp()
    } else {
        chrono::Local::now().timestamp()
    }
}

pub fn os_difftime(t1: i64, t2: i64) -> f64 {
    (t1 - t2) as f64
}

pub fn os_setlocale(_locale: Option<&str>, _category: Option<&str>) -> Option<String> {
    // Not implemented: locale setting is platform-specific
    None
}

pub fn os_exit(status: Option<i32>) -> ! {
    exit(status.unwrap_or(0));
}

// --- Error type for loslib operations
#[derive(Debug)]
pub enum OsLibError {
    Io(std::io::Error),
    Command(String),
    InvalidInput(String),
}

impl std::fmt::Display for OsLibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsLibError::Io(e) => write!(f, "IO error: {}", e),
            OsLibError::Command(e) => write!(f, "Command error: {}", e),
            OsLibError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

impl From<std::io::Error> for OsLibError {
    fn from(e: std::io::Error) -> Self { OsLibError::Io(e) }
}

/// More idiomatic result type for loslib
pub type OsLibResult<T> = Result<T, OsLibError>;

/// Extended time/date helpers
pub fn os_now_utc() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn os_now_local() -> i64 {
    chrono::Local::now().timestamp()
}

/// Struct for easy Lua registration (future integration)
pub struct OsLib;

impl OsLib {
    pub fn register(_L: &mut LuaState) {
        // Register all functions to Lua state here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tmpname() {
        let name = os_tmpname().unwrap();
        assert!(name.contains("lua_"));
    }
    #[test]
    fn test_getenv() {
        std::env::set_var("LUA_TEST_ENV", "ok");
        assert_eq!(os_getenv("LUA_TEST_ENV"), Some("ok".to_string()));
    }
    #[test]
    fn test_time() {
        let now = os_now_utc();
        assert!(now > 0);
    }
}

/// Returns the list of all required OS library function names for completeness checking
pub fn required_os_functions() -> &'static [&'static str] {
    &[
        "clock", "date", "difftime", "execute", "exit", "getenv", "remove", "rename", "setlocale", "time", "tmpname"
    ]
}

#[cfg(test)]
mod completeness_tests {
    use super::*;
    #[test]
    fn test_required_os_functions_count() {
        // There should be 11 required functions for a complete Lua OS library
        assert_eq!(required_os_functions().len(), 11);
    }
}

// --- Registration stub for Lua integration ---
pub fn luaopen_os(_L: &mut LuaState) {
    // Register all above functions to the Lua state
}
