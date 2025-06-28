// lmathlib.rs
// Rust implementation of the Lua standard math library for Lua Skylet

use std::f64::consts::PI;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use crate::lapi::{luaL_checknumber, luaL_register, lua_pushnumber, lua_pushinteger, luaL_checkinteger, lua_pushboolean, lua_isnumber, lua_tonumber, luaL_error};
use crate::lua_State;

// Helper macro for defining math functions with single number argument
macro_rules! math_unary_fn {
    ($fn_name:ident, $func:expr) => {
        extern "C" fn $fn_name(L: *mut lua_State) -> c_int {
            unsafe {
                let x = luaL_checknumber(L, 1);
                lua_pushnumber(L, $func(x));
                1
            }
        }
    };
}

// Helper macro for defining math functions with two number arguments
macro_rules! math_binary_fn {
    ($fn_name:ident, $func:expr) => {
        extern "C" fn $fn_name(L: *mut lua_State) -> c_int {
            unsafe {
                let x = luaL_checknumber(L, 1);
                let y = luaL_checknumber(L, 2);
                lua_pushnumber(L, $func(x, y));
                1
            }
        }
    };
}

// math.abs
math_unary_fn!(math_abs, |x: f64| x.abs());

// math.sin
math_unary_fn!(math_sin, |x: f64| x.sin());

// math.cos
math_unary_fn!(math_cos, |x: f64| x.cos());

// math.tan
math_unary_fn!(math_tan, |x: f64| x.tan());

// math.asin
math_unary_fn!(math_asin, |x: f64| x.asin());

// math.acos
math_unary_fn!(math_acos, |x: f64| x.acos());

// math.atan
extern "C" fn math_atan(L: *mut lua_State) -> c_int {
    unsafe {
        let x = luaL_checknumber(L, 1);
        if lua_isnumber(L, 2) != 0 {
            let y = luaL_checknumber(L, 2);
            lua_pushnumber(L, x.atan2(y));
            1
        } else {
            lua_pushnumber(L, x.atan());
            1
        }
    }
}

// math.ceil
math_unary_fn!(math_ceil, |x: f64| x.ceil());

// math.floor
math_unary_fn!(math_floor, |x: f64| x.floor());

// math.sqrt
math_unary_fn!(math_sqrt, |x: f64| x.sqrt());

// math.log
extern "C" fn math_log(L: *mut lua_State) -> c_int {
    unsafe {
        let x = luaL_checknumber(L, 1);
        let base = if lua_isnumber(L, 2) != 0 {
            luaL_checknumber(L, 2)
        } else {
            std::f64::consts::E
        };
        lua_pushnumber(L, x.log(base));
        1
    }
}

// math.exp
math_unary_fn!(math_exp, |x: f64| x.exp());

// math.pow (alias for math.pow)
math_binary_fn!(math_pow, |x: f64, y: f64| x.powf(y));

// math.max
extern "C" fn math_max(L: *mut lua_State) -> c_int {
    unsafe {
        let n = crate::lapi::lua_gettop(L);
        let mut max_val = luaL_checknumber(L, 1);
        for i in 2..=n {
            let val = luaL_checknumber(L, i);
            if val > max_val {
                max_val = val;
            }
        }
        lua_pushnumber(L, max_val);
        1
    }
}

// math.min
extern "C" fn math_min(L: *mut lua_State) -> c_int {
    unsafe {
        let n = crate::lapi::lua_gettop(L);
        let mut min_val = luaL_checknumber(L, 1);
        for i in 2..=n {
            let val = luaL_checknumber(L, i);
            if val < min_val {
                min_val = val;
            }
        }
        lua_pushnumber(L, min_val);
        1
    }
}

// math.random
extern "C" fn math_random(L: *mut lua_State) -> c_int {
    unsafe {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n = crate::lapi::lua_gettop(L);
        match n {
            0 => {
                lua_pushnumber(L, rng.gen::<f64>());
                1
            }
            1 => {
                let upper = luaL_checkinteger(L, 1);
                if upper < 1 {
                    return luaL_error(L, "interval is empty".as_ptr() as *const c_char);
                }
                let r = rng.gen_range(1..=upper);
                lua_pushinteger(L, r);
                1
            }
            2 => {
                let lower = luaL_checkinteger(L, 1);
                let upper = luaL_checkinteger(L, 2);
                if lower > upper {
                    return luaL_error(L, "interval is empty".as_ptr() as *const c_char);
                }
                let r = rng.gen_range(lower..=upper);
                lua_pushinteger(L, r);
                1
            }
            _ => luaL_error(L, "wrong number of arguments".as_ptr() as *const c_char),
        }
    }
}

// math.randomseed
extern "C" fn math_randomseed(L: *mut lua_State) -> c_int {
    unsafe {
        use rand::{SeedableRng, rngs::StdRng};
        let seed = luaL_checkinteger(L, 1) as u64;
        let _rng = StdRng::seed_from_u64(seed);
        // Note: You must store _rng somewhere to use later random calls
        0
    }
}

// math type table
static MATH_LIB: &[(&str, unsafe extern "C" fn(*mut lua_State) -> c_int)] = &[
    ("abs", math_abs),
    ("sin", math_sin),
    ("cos", math_cos),
    ("tan", math_tan),
    ("asin", math_asin),
    ("acos", math_acos),
    ("atan", math_atan),
    ("ceil", math_ceil),
    ("floor", math_floor),
    ("sqrt", math_sqrt),
    ("log", math_log),
    ("exp", math_exp),
    ("pow", math_pow),
    ("max", math_max),
    ("min", math_min),
    ("random", math_random),
    ("randomseed", math_randomseed),
];

// Register math library in Lua
#[no_mangle]
pub unsafe extern "C" fn luaopen_math(L: *mut lua_State) -> c_int {
    luaL_register(L, cstr!("math"), MATH_LIB);
    // Push math.pi constant
    lua_pushnumber(L, PI);
    lua_setfield(L, -2, cstr!("pi"));
    1
}
