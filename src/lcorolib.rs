//! lcorolib.rs
//! Coroutine library for Lua Skylet (Rust version).
//! Provides coroutine.create, coroutine.resume, coroutine.yield, coroutine.status, coroutine.wrap, coroutine.yieldable.

use crate::lapi::*;
use crate::lobject::*;
use crate::lstate::*;
use std::os::raw::{c_int, c_void};

/// Coroutine status codes modeled after Lua's
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
pub enum CoroutineStatus {
    Ok = 0,
    Yield = 1,
    Error = 2,
}

/// coroutine.create(f)
/// Creates a new coroutine running function `f`.
/// Returns the new coroutine thread.
#[no_mangle]
pub unsafe extern "C" fn luaB_cocreate(L: *mut lua_State) -> c_int {
    luaL_checktype(L, 1, LUA_TFUNCTION); // ensure argument is function
    let co = lua_newthread(L);           // create new coroutine thread
    lua_pushvalue(L, 1);                 // push function onto stack
    lua_xmove(L, co, 1);                 // move function to coroutine stack
    // coroutine thread is already on stack as return value
    1
}

/// coroutine.resume(co, ...)
/// Resumes coroutine `co` with arguments.
/// Returns: true + results on success, false + error message on failure.
#[no_mangle]
pub unsafe extern "C" fn luaB_coresume(L: *mut lua_State) -> c_int {
    let co = lua_tothread(L, 1);
    if co.is_null() {
        lua_pushboolean(L, 0);
        lua_pushstring(L, cstr!("bad argument #1 (coroutine expected)"));
        return 2;
    }
    let status = lua_status(co);
    if status != LUA_YIELD && status != LUA_OK {
        lua_pushboolean(L, 0);
        lua_pushstring(L, cstr!("cannot resume dead coroutine"));
        return 2;
    }
    let nargs = lua_gettop(L) - 1;
    lua_xmove(L, co, nargs);
    let status = lua_resume(co, L, nargs);
    if status == LUA_OK || status == LUA_YIELD {
        lua_pushboolean(L, 1);
        let nresults = lua_gettop(co);
        lua_xmove(co, L, nresults);
        return (nresults + 1) as c_int;
    } else {
        lua_pushboolean(L, 0);
        // Push error message from coroutine stack
        if lua_gettop(co) > 0 {
            lua_xmove(co, L, 1);
        } else {
            lua_pushstring(L, cstr!("coroutine error"));
        }
        return 2;
    }
}

/// coroutine.yield(...)
/// Yields the running coroutine, returning values to the resumer.
#[no_mangle]
pub unsafe extern "C" fn luaB_yield(L: *mut lua_State) -> c_int {
    let n = lua_gettop(L);
    lua_yield(L, n)
}

/// coroutine.status(co)
/// Returns the status string of a coroutine: "running", "suspended", "normal", or "dead".
#[no_mangle]
pub unsafe extern "C" fn luaB_costatus(L: *mut lua_State) -> c_int {
    let co = lua_tothread(L, 1);
    if co.is_null() {
        luaL_error(L, cstr!("bad argument #1 (coroutine expected)"));
        return 0; // unreachable
    }
    let status = lua_status(co);
    let status_str = if co == lua_pushthread(L) {
        // running coroutine
        lua_pop(L, 1);
        "running"
    } else {
        match status {
            LUA_YIELD => "suspended",
            LUA_OK => {
                // If stack is empty, coroutine is dead
                if lua_gettop(co) == 0 {
                    "dead"
                } else {
                    "normal"
                }
            }
            _ => "dead",
        }
    };
    lua_pushstring(L, cstr!(status_str));
    1
}

/// coroutine.wrap(f)
/// Returns a function that resumes the coroutine created from `f`.
#[no_mangle]
pub unsafe extern "C" fn luaB_cowrap(L: *mut lua_State) -> c_int {
    luaB_cocreate(L); // pushes coroutine thread
    lua_pushcclosure(L, Some(luaB_auxwrap), 1); // closure with coroutine as upvalue
    1
}

/// Auxiliary function used by `coroutine.wrap`.
unsafe extern "C" fn luaB_auxwrap(L: *mut lua_State) -> c_int {
    let co = lua_tothread(L, lua_upvalueindex(1));
    let nargs = lua_gettop(L);
    lua_xmove(L, co, nargs);
    let status = lua_resume(co, L, nargs);
    if status == LUA_OK || status == LUA_YIELD {
        let nresults = lua_gettop(co);
        lua_xmove(co, L, nresults);
        return nresults as c_int;
    } else {
        // propagate error as Lua error
        if lua_gettop(co) > 0 {
            lua_xmove(co, L, 1);
        } else {
            lua_pushstring(L, cstr!("error in coroutine wrap"));
        }
        lua_error(L);
        unreachable!();
    }
}

/// coroutine.yieldable()
/// Returns true if the running coroutine can yield.
#[no_mangle]
pub unsafe extern "C" fn lua_yieldable(L: *mut lua_State) -> c_int {
    let yieldable = lua_isyieldable(L);
    lua_pushboolean(L, if yieldable != 0 { 1 } else { 0 });
    1
}

/// Creates the coroutine library table and registers functions.
pub unsafe fn luaopen_coroutine(L: *mut lua_State) -> c_int {
    lua_newtable(L);

    // Register coroutine functions
    lua_pushcfunction(L, Some(luaB_cocreate));
    lua_setfield(L, -2, cstr!("create"));

    lua_pushcfunction(L, Some(luaB_coresume));
    lua_setfield(L, -2, cstr!("resume"));

    lua_pushcfunction(L, Some(luaB_yield));
    lua_setfield(L, -2, cstr!("yield"));

    lua_pushcfunction(L, Some(luaB_costatus));
    lua_setfield(L, -2, cstr!("status"));

    lua_pushcfunction(L, Some(luaB_cowrap));
    lua_setfield(L, -2, cstr!("wrap"));

    lua_pushcfunction(L, Some(lua_yieldable));
    lua_setfield(L, -2, cstr!("yieldable"));

    1
}