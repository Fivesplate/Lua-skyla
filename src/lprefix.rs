//! lprefix.rs - Pre-header configuration for Lua (Rust port)
// This module provides platform and feature flags that must be set before any other module.

#[cfg(not(feature = "c89"))]
pub const XOPEN_SOURCE: Option<u32> = Some(600);

#[cfg(all(not(feature = "c89"), not(feature = "lua_32bits")))]
pub const LARGEFILE_SOURCE: bool = true;
#[cfg(all(not(feature = "c89"), not(feature = "lua_32bits")))]
pub const FILE_OFFSET_BITS: u32 = 64;

#[cfg(windows)]
pub const CRT_SECURE_NO_WARNINGS: bool = true;

// Usage: import this module at the top of any file that needs these flags.
// These constants can be used for conditional compilation or platform-specific logic.
