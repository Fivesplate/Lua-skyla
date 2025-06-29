//! ltablib.rs - Lua standard table library for Rust-based Lua VM
// Ported and adapted from ltablib.c
/*
** $Id: ltablib.c $
** Library for Table Manipulation
** See Copyright Notice in lua.h
*/

// --- Module flags and helpers (ported from C defines) ---
const TAB_R: u8 = 1; // read
const TAB_W: u8 = 2; // write
const TAB_L: u8 = 4; // length
const TAB_RW: u8 = TAB_R | TAB_W; // read/write

// Custom unimplemented macro for this module
macro_rules! unimplemented_table {
    ($name:expr) => {{
        eprintln!("[ltablib] function '{}' is not yet implemented", $name);
        // You may want to return a Lua error or panic here
        // For now, just panic for visibility
        panic!("[ltablib] function '{}' is not yet implemented", $name);
    }};
}

use crate::lstate::LuaState;
use crate::lobject::LuaValue;

// Helper: checkfield
fn checkfield(state: &mut LuaState, key: &str, n: i32) -> bool {
    // Push the string key
    state.push(LuaValue::Str(key.to_string()));
    // Raw get from the table at stack index -n
    let result = state.raw_get(-n);
    // Check if the result is not nil
    let is_not_nil = !matches!(result, LuaValue::Nil);
    // Pop the result from the stack if needed (depends on your API)
    state.pop(1);
    is_not_nil
}

// Helper: aux_getn
fn aux_getn(state: &mut LuaState, n: i32, w: u8) -> i64 {
    // This would check the table and get its length
    // In C: (checktab(L, n, (w) | TAB_L), luaL_len(L, n))
    // Here, we assume checktab is handled elsewhere or not needed in Rust
    state.len(n)
}

// Register all table library functions
pub fn open_table_lib(state: &mut LuaState) {
    // Register each function below with the global 'table' library
    // Example: state.register_lib_function("table", "concat", table_concat);
}

// table.concat(table, sep, i, j)
pub fn table_concat(state: &mut LuaState) -> i32 {
    let table = state.check_table(1);
    let sep = state.opt_string(2, "");
    let i = state.opt_integer(3, 1);
    let j = state.opt_integer(4, aux_getn(state, 1, TAB_R));
    let mut result = String::new();
    for idx in i..=j {
        let v = table.get(idx as usize);
        match v {
            LuaValue::Str(ref s) => {
                if idx > i {
                    result.push_str(&sep);
                }
                result.push_str(s);
            }
            _ => {
                state.error(&format!("invalid value at index {} in table for 'concat'", idx));
                return 0;
            }
        }
    }
    state.push(LuaValue::Str(result));
    1
}

// table.insert(table, [pos,] value)
pub fn table_insert(state: &mut LuaState) -> i32 {
    // Get the number of arguments
    let nargs = state.get_top();
    // Get the table
    let table = state.check_table(1);
    let len = aux_getn(state, 1, TAB_RW);
    let mut pos = len + 1; // default: insert at end
    let value;
    if nargs == 2 {
        value = state.to_value(2);
    } else if nargs == 3 {
        pos = state.check_integer(2);
        value = state.to_value(3);
        // Check bounds
        if pos < 1 || pos > len + 1 {
            state.arg_error(2, "position out of bounds");
        }
        // Move up elements
        for i in (pos..=len).rev() {
            let v = table.get(i as usize);
            table.set((i + 1) as usize, v);
        }
    } else {
        state.error("wrong number of arguments to 'insert'");
        return 0;
    }
    table.set(pos as usize, value);
    0
}

// table.remove(table, [pos])
pub fn table_remove(state: &mut LuaState) -> i32 {
    let table = state.check_table(1);
    let len = aux_getn(state, 1, TAB_RW);
    let pos = state.opt_integer(2, len);
    if pos != len {
        if pos < 1 || pos > len {
            state.arg_error(2, "position out of bounds");
        }
    }
    let result = table.get(pos as usize);
    for i in pos..len {
        let v = table.get((i + 1) as usize);
        table.set(i as usize, v);
    }
    table.set(len as usize, LuaValue::Nil);
    state.push(result);
    1
}

// table.move(a1, f, e, t [,a2])
pub fn table_move(state: &mut LuaState) -> i32 {
    let f = state.check_integer(2);
    let e = state.check_integer(3);
    let t = state.check_integer(4);
    let tt = if state.is_none_or_nil(5) { 1 } else { 5 };
    let src = state.check_table(1);
    let dst = state.check_table(tt);
    if e >= f {
        let n = e - f + 1;
        if t > i64::MAX - n + 1 {
            state.arg_error(4, "destination wrap around");
        }
        if t > e || t <= f || (tt != 1 && !state.compare_tables(1, tt)) {
            for i in 0..n {
                let v = src.get((f + i) as usize);
                dst.set((t + i) as usize, v);
            }
        } else {
            for i in (0..n).rev() {
                let v = src.get((f + i) as usize);
                dst.set((t + i) as usize, v);
            }
        }
    }
    state.push(dst.clone());
    1
}

// table.pack(...)
pub fn table_pack(state: &mut LuaState) -> i32 {
    let n = state.get_top();
    let table = state.create_table(n, 1);
    for i in 1..=n {
        let v = state.to_value(i);
        table.set(i, v);
    }
    table.set_field("n", LuaValue::Int(n as i64));
    state.push(table);
    1
}

// table.unpack(list, [i, j])
pub fn table_unpack(state: &mut LuaState) -> i32 {
    let i = state.opt_integer(2, 1);
    let e = state.opt_integer(3, aux_getn(state, 1, TAB_R));
    if i > e {
        return 0;
    }
    let table = state.check_table(1);
    let mut n = 0;
    for idx in i..=e {
        let v = table.get(idx as usize);
        state.push(v);
        n += 1;
    }
    n
}

// table.sort(table [, comp])
pub fn table_sort(state: &mut LuaState) -> i32 {
    // TODO: Implement full sort logic with optional comparator
    unimplemented_table!("table.sort");
}

// table.create(sizeseq, sizerest)
pub fn table_create(state: &mut LuaState) -> i32 {
    // Get arguments (default sizerest = 0)
    let sizeseq = state.check_integer(1).max(0) as usize;
    let sizerest = state.opt_integer(2, 0).max(0) as usize;
    // Optionally check for overflow (INT_MAX)
    // Create a new table with the given capacities
    let table = state.create_table(sizeseq, sizerest);
    state.push(table);
    1
}