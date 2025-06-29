// skylaconf.rs - Skyla/Lua configuration (Rust translation of luaconf.h)
// This file centralizes all VM configuration, numeric types, paths, and feature flags.
// Platform-specific logic uses Rust's cfg! macros. Adjust as needed for your build.

use std::env;
use std::ops::{Add, Sub, Mul, Div};

// === System/Platform Configuration ===
#[cfg(windows)]
pub const USE_WINDOWS: bool = true;
#[cfg(not(windows))]
pub const USE_WINDOWS: bool = false;

// === Numeric Types ===
// Integer type
#[cfg(feature = "int32")]
pub type LuaInteger = i32;
#[cfg(all(not(feature = "int32"), feature = "int64"))]
pub type LuaInteger = i64;
#[cfg(all(not(feature = "int32"), not(feature = "int64")))]
pub type LuaInteger = i64; // default

// Float type
#[cfg(feature = "float32")]
pub type LuaFloat = f32;
#[cfg(all(not(feature = "float32"), feature = "float64"))]
pub type LuaFloat = f64;
#[cfg(all(not(feature = "float32"), not(feature = "float64")))]
pub type LuaFloat = f64; // default

// === Numeric Limits ===
pub const LUA_INTEGER_MIN: LuaInteger = LuaInteger::MIN;
pub const LUA_INTEGER_MAX: LuaInteger = LuaInteger::MAX;
#[allow(dead_code)]
pub const LUA_FLOAT_MIN: LuaFloat = LuaFloat::MIN;
pub const LUA_FLOAT_MAX: LuaFloat = LuaFloat::MAX;

// === Version and Build Info ===
pub const SKYLA_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SKYLA_BUILD_PROFILE: &str = env!("PROFILE");

// === Feature Flags ===
#[cfg(feature = "api_check")]
pub const USE_API_CHECK: bool = true;
#[cfg(not(feature = "api_check"))]
pub const USE_API_CHECK: bool = false;

#[cfg(feature = "nocvtn2s")]
pub const NOCVTN2S: bool = true; // No number-to-string coercion
#[cfg(not(feature = "nocvtn2s"))]
pub const NOCVTN2S: bool = false;

#[cfg(feature = "nocvts2n")]
pub const NOCVTS2N: bool = true; // No string-to-number coercion
#[cfg(not(feature = "nocvts2n"))]
pub const NOCVTS2N: bool = false;

// === Path Configuration ===
#[cfg(windows)]
pub const PATH_SEP: &str = ";";
#[cfg(windows)]
pub const DIR_SEP: &str = "\\";
#[cfg(not(windows))]
pub const PATH_SEP: &str = ":";
#[cfg(not(windows))]
pub const DIR_SEP: &str = "/";
pub const PATH_MARK: &str = "?";
pub const EXEC_DIR: &str = "!";
pub const IG_MARK: &str = "-";

// Default search paths (adjust as needed)
#[cfg(windows)]
pub const LUA_PATH_DEFAULT: &str = "!\\lua\\?.lua;!\\lua\\?\\init.lua;!\\?.lua;!\\?\\init.lua;.\\?.lua;.\\?\\init.lua";
#[cfg(windows)]
pub const LUA_CPATH_DEFAULT: &str = "!\\?.dll;!\\..\\lib\\lua\\?.dll;!\\loadall.dll;.\\?.dll";
#[cfg(not(windows))]
pub const LUA_PATH_DEFAULT: &str = "/usr/local/share/lua/?.lua;/usr/local/share/lua/?/init.lua;/usr/local/lib/lua/?.lua;/usr/local/lib/lua/?/init.lua;./?.lua;./?/init.lua";
#[cfg(not(windows))]
pub const LUA_CPATH_DEFAULT: &str = "/usr/local/lib/lua/?.so;/usr/local/lib/lua/loadall.so;./?.so";

// === Stack/Buffer Sizes ===
pub const MAX_STACK: usize = 1000000;
pub const EXTRASPACE: usize = std::mem::size_of::<*const ()>();
pub const IDSIZE: usize = 60;
pub const LUAL_BUFFERSIZE: usize = 16 * std::mem::size_of::<*const ()>() * std::mem::size_of::<LuaFloat>();

// === Compatibility/Feature Flags ===
pub const COMPAT_GLOBAL: bool = true;
pub const COMPAT_5_3: bool = true;
pub const COMPAT_MATHLIB: bool = true;
pub const COMPAT_APIINTCASTS: bool = true;
pub const COMPAT_LT_LE: bool = true;

// === API Visibility (no-op in Rust, for reference) ===
// pub use visibility as needed

// === Config Introspection ===
pub fn print_config() {
    println!("Skyla/Lua Config:");
    println!("  Version: {}", SKYLA_VERSION);
    println!("  Build: {}", SKYLA_BUILD_PROFILE);
    println!("  Integer: {} (min={}, max={})", std::any::type_name::<LuaInteger>(), LUA_INTEGER_MIN, LUA_INTEGER_MAX);
    println!("  Float: {} (min={}, max={})", std::any::type_name::<LuaFloat>(), LUA_FLOAT_MIN, LUA_FLOAT_MAX);
    println!("  Path sep: {}  Dir sep: {}", PATH_SEP, DIR_SEP);
    println!("  Lua path: {}", LUA_PATH_DEFAULT);
    println!("  C path: {}", LUA_CPATH_DEFAULT);
    println!("  Max stack: {}  Buffer size: {}", MAX_STACK, LUAL_BUFFERSIZE);
    println!("  API check: {}  NOCVTN2S: {}  NOCVTS2N: {}", USE_API_CHECK, NOCVTN2S, NOCVTS2N);
    println!("  Compat: global={}  5.3={}  mathlib={}  apiintcasts={}  lt_le={}", COMPAT_GLOBAL, COMPAT_5_3, COMPAT_MATHLIB, COMPAT_APIINTCASTS, COMPAT_LT_LE);
}

// === Local configuration space ===
// Add custom project-specific config here

// === Config Struct and Serialization (for tests, plugins, debugging) ===
#[derive(Debug, Clone)]
pub struct SkylaConfig {
    pub version: &'static str,
    pub build_profile: &'static str,
    pub integer_type: &'static str,
    pub integer_min: LuaInteger,
    pub integer_max: LuaInteger,
    pub float_type: &'static str,
    pub float_min: LuaFloat,
    pub float_max: LuaFloat,
    pub path_sep: &'static str,
    pub dir_sep: &'static str,
    pub lua_path: &'static str,
    pub c_path: &'static str,
    pub max_stack: usize,
    pub buffer_size: usize,
    pub api_check: bool,
    pub nocvtn2s: bool,
    pub nocvts2n: bool,
    pub compat_global: bool,
    pub compat_53: bool,
    pub compat_mathlib: bool,
    pub compat_apiintcasts: bool,
    pub compat_lt_le: bool,
    pub fuzzing: bool,
    pub snapshot: bool,
    pub plugin_hooks: bool,
}

impl SkylaConfig {
    pub fn current() -> Self {
        SkylaConfig {
            version: SKYLA_VERSION,
            build_profile: SKYLA_BUILD_PROFILE,
            integer_type: std::any::type_name::<LuaInteger>(),
            integer_min: LUA_INTEGER_MIN,
            integer_max: LUA_INTEGER_MAX,
            float_type: std::any::type_name::<LuaFloat>(),
            float_min: LUA_FLOAT_MIN,
            float_max: LUA_FLOAT_MAX,
            path_sep: PATH_SEP,
            dir_sep: DIR_SEP,
            lua_path: LUA_PATH_DEFAULT,
            c_path: LUA_CPATH_DEFAULT,
            max_stack: MAX_STACK,
            buffer_size: LUAL_BUFFERSIZE,
            api_check: USE_API_CHECK,
            nocvtn2s: NOCVTN2S,
            nocvts2n: NOCVTS2N,
            compat_global: COMPAT_GLOBAL,
            compat_53: COMPAT_5_3,
            compat_mathlib: COMPAT_MATHLIB,
            compat_apiintcasts: COMPAT_APIINTCASTS,
            compat_lt_le: COMPAT_LT_LE,
            fuzzing: option_env!("SKYLA_FUZZ").is_some(),
            snapshot: option_env!("SKYLA_SNAPSHOT").is_some(),
            plugin_hooks: option_env!("SKYLA_PLUGINS").is_some(),
        }
    }
    #[cfg(feature = "serde")] // Optional: enable with serde
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<serialization error>".to_string())
    }
}

// === Macro for marking deprecated/compat APIs ===
#[macro_export]
macro_rules! skyla_deprecated {
    ($msg:expr) => {
        #[deprecated(note = $msg)]
    };
}

// === Environment Variable Defaults ===
pub const ENV_DEBUG: &str = "SKYLA_DEBUG";
pub const ENV_GOODBYE: &str = "SKYLA_GOODBYE";
pub const ENV_FUZZ: &str = "SKYLA_FUZZ";
pub const ENV_SNAPSHOT: &str = "SKYLA_SNAPSHOT";
pub const ENV_PLUGINS: &str = "SKYLA_PLUGINS";

// === Experimental/Advanced Feature Flags ===
#[cfg(feature = "deterministic_fuzzing")]
pub const DETERMINISTIC_FUZZING: bool = true;
#[cfg(not(feature = "deterministic_fuzzing"))]
pub const DETERMINISTIC_FUZZING: bool = false;

#[cfg(feature = "coverage")]
pub const COVERAGE: bool = true;
#[cfg(not(feature = "coverage"))]
pub const COVERAGE: bool = false;

#[cfg(feature = "invariant_check")]
pub const INVARIANT_CHECK: bool = true;
#[cfg(not(feature = "invariant_check"))]
pub const INVARIANT_CHECK: bool = false;

// === Platform/Build Info Utilities ===
/// Returns a string describing the current platform and build info.
pub fn platform_info() -> String {
    format!(
        "Skyla {} [{}] on {} (Rust {})",
        SKYLA_VERSION,
        SKYLA_BUILD_PROFILE,
        if USE_WINDOWS { "windows" } else { "unix" },
        rustc_version_runtime::version()
    )
}

/// Print a summary of all enabled experimental features.
pub fn print_experimental_features() {
    println!("Experimental/Advanced Features:");
    println!("  Deterministic fuzzing: {}", DETERMINISTIC_FUZZING);
    println!("  Coverage: {}", COVERAGE);
    println!("  Invariant check: {}", INVARIANT_CHECK);
    println!("  Fuzzing (env): {}", option_env!("SKYLA_FUZZ").is_some());
    println!("  Snapshot (env): {}", option_env!("SKYLA_SNAPSHOT").is_some());
    println!("  Plugin hooks (env): {}", option_env!("SKYLA_PLUGINS").is_some());
}

// === Utility: Print a summary of all config, platform, and features ===
pub fn print_full_config() {
    println!("{}", platform_info());
    print_config();
    print_experimental_features();
    #[cfg(feature = "serde")]
    print_config_json();
    #[cfg(not(feature = "serde"))]
    print_config_debug();
}

// === Utility: Print a minimal config summary (for CI/logging) ===
pub fn print_minimal_config() {
    println!(
        "Skyla {} [{}] | Int: {} | Float: {} | Stack: {} | Fuzz: {} | Plugins: {}",
        SKYLA_VERSION,
        SKYLA_BUILD_PROFILE,
        std::any::type_name::<LuaInteger>(),
        std::any::type_name::<LuaFloat>(),
        MAX_STACK,
        option_env!("SKYLA_FUZZ").is_some(),
        option_env!("SKYLA_PLUGINS").is_some()
    );
}

// === Example: Configurable warning for experimental features ===
pub fn warn_if_experimental() {
    if DETERMINISTIC_FUZZING || COVERAGE || INVARIANT_CHECK {
        eprintln!("[skyla] Warning: Experimental features enabled!");
    }
}

// === Example: Function to check if a feature is enabled by name ===
pub fn is_feature_enabled(name: &str) -> bool {
    match name.to_ascii_lowercase().as_str() {
        "deterministic_fuzzing" => DETERMINISTIC_FUZZING,
        "coverage" => COVERAGE,
        "invariant_check" => INVARIANT_CHECK,
        "fuzzing_env" => option_env!("SKYLA_FUZZ").is_some(),
        "snapshot_env" => option_env!("SKYLA_SNAPSHOT").is_some(),
        "plugin_hooks_env" => option_env!("SKYLA_PLUGINS").is_some(),
        _ => false,
    }
}

// === Example: Runtime assertion for config invariants ===
pub fn assert_config_sanity() {
    assert!(MAX_STACK > 1000, "MAX_STACK must be > 1000");
    assert!(LUAL_BUFFERSIZE > 0, "LUAL_BUFFERSIZE must be > 0");
    // Add more runtime checks as needed
}

// === Example: Feature summary as a struct (for tests/plugins) ===
#[derive(Debug, Clone)]
pub struct SkylaFeatureSummary {
    pub deterministic_fuzzing: bool,
    pub coverage: bool,
    pub invariant_check: bool,
    pub fuzzing_env: bool,
    pub snapshot_env: bool,
    pub plugin_hooks_env: bool,
}

impl SkylaFeatureSummary {
    pub fn current() -> Self {
        Self {
            deterministic_fuzzing: DETERMINISTIC_FUZZING,
            coverage: COVERAGE,
            invariant_check: INVARIANT_CHECK,
            fuzzing_env: option_env!("SKYLA_FUZZ").is_some(),
            snapshot_env: option_env!("SKYLA_SNAPSHOT").is_some(),
            plugin_hooks_env: option_env!("SKYLA_PLUGINS").is_some(),
        }
    }
}

// === Example: Compile-time assertion for config sanity ===
#[allow(dead_code)]
const _: () = {
    assert!(MAX_STACK <= 2_000_000, "MAX_STACK too large");
};

// === Example: Helper for environment variable with fallback ===
pub fn get_env_or_default(var: &str, default: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| default.to_string())
}

// === Utility: Print all config as JSON (if serde enabled) or as debug ===
#[cfg(feature = "serde")]
pub fn print_config_json() {
    let config = SkylaConfig::current();
    println!("{}", config.to_json());
}

#[cfg(not(feature = "serde"))]
pub fn print_config_debug() {
    let config = SkylaConfig::current();
    println!("{:?}", config);
}

// === Example: Use this macro to mark deprecated functions ===
// skyla_deprecated!("Use new_foo instead");

// === Local project-specific toggles ===
// pub const ENABLE_MY_FEATURE: bool = true;

// End of hyper-extended skylaconf.rs
