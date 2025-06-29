//! ltm.rs - Tag methods (metamethods) for Rust-based Lua VM
// Ported and modernized from ltm.c/h

use crate::lobject::{LuaValue, GcObject, LuaTable, LuaString};
use crate::lstate::LuaState;
use std::sync::Arc;

/// Enumeration of all Lua metamethods (ORDER TM)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TMS {
    Index,
    NewIndex,
    Gc,
    Mode,
    Len,
    Eq, // last fast-access
    Add,
    Sub,
    Mul,
    Mod,
    Pow,
    Div,
    IDiv,
    Band,
    Bor,
    Bxor,
    Shl,
    Shr,
    Unm,
    Bnot,
    Lt,
    Le,
    Concat,
    Call,
    Close,
    N, // number of elements
}

impl TMS {
    pub const COUNT: usize = 24; // TM_N (not including N itself)
    pub fn as_usize(self) -> usize { self as usize }
    pub fn from_usize(i: usize) -> Option<TMS> {
        use TMS::*;
        match i {
            0 => Some(Index), 1 => Some(NewIndex), 2 => Some(Gc), 3 => Some(Mode),
            4 => Some(Len), 5 => Some(Eq), 6 => Some(Add), 7 => Some(Sub),
            8 => Some(Mul), 9 => Some(Mod), 10 => Some(Pow), 11 => Some(Div),
            12 => Some(IDiv), 13 => Some(Band), 14 => Some(Bor), 15 => Some(Bxor),
            16 => Some(Shl), 17 => Some(Shr), 18 => Some(Unm), 19 => Some(Bnot),
            20 => Some(Lt), 21 => Some(Le), 22 => Some(Concat), 23 => Some(Call),
            24 => Some(Close), _ => None
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            TMS::Index => "__index",
            TMS::NewIndex => "__newindex",
            TMS::Gc => "__gc",
            TMS::Mode => "__mode",
            TMS::Len => "__len",
            TMS::Eq => "__eq",
            TMS::Add => "__add",
            TMS::Sub => "__sub",
            TMS::Mul => "__mul",
            TMS::Mod => "__mod",
            TMS::Pow => "__pow",
            TMS::Div => "__div",
            TMS::IDiv => "__idiv",
            TMS::Band => "__band",
            TMS::Bor => "__bor",
            TMS::Bxor => "__bxor",
            TMS::Shl => "__shl",
            TMS::Shr => "__shr",
            TMS::Unm => "__unm",
            TMS::Bnot => "__bnot",
            TMS::Lt => "__lt",
            TMS::Le => "__le",
            TMS::Concat => "__concat",
            TMS::Call => "__call",
            TMS::Close => "__close",
            TMS::N => "<invalid>"
        }
    }
}

/// Type names for Lua types (for error messages, etc.)
pub const LUA_TYPE_NAMES: [&str; 11] = [
    "no value", "nil", "boolean", "userdata", "number",
    "string", "table", "function", "userdata", "thread", "upvalue"
];

/// Lookup a metamethod in a table's metatable
pub fn get_tm(table: &LuaTable, event: TMS) -> Option<LuaValue> {
    table.get_metatable().and_then(|mt| mt.get(&LuaValue::Str(event.name().to_string())))
}

/// Fast path: check if metatable is missing the metamethod (using flags)
pub fn has_no_tm(table: &LuaTable, event: TMS) -> bool {
    // In a real implementation, use table flags for fast path
    // Here, just check if the metatable is missing the field
    table.get_metatable().map_or(true, |mt| !mt.contains_key(&LuaValue::Str(event.name().to_string())))
}

/// Call a metamethod (generic)
pub fn call_tm(state: &mut LuaState, f: &LuaValue, args: &[LuaValue]) -> Option<LuaValue> {
    // In a real implementation, push args and call function in VM
    // Here, just a stub
    let _ = (state, f, args);
    None
}

/// Try binary metamethod (e.g., __add, __sub)
pub fn try_bin_tm(state: &mut LuaState, a: &LuaValue, b: &LuaValue, event: TMS) -> Option<LuaValue> {
    let mt_a = a.get_metatable();
    let mt_b = b.get_metatable();
    let mm = mt_a.and_then(|mt| mt.get(&LuaValue::Str(event.name().to_string())))
        .or_else(|| mt_b.and_then(|mt| mt.get(&LuaValue::Str(event.name().to_string()))));
    mm.and_then(|f| call_tm(state, &f, &[a.clone(), b.clone()]))
}

/// Try order metamethod (e.g., __lt, __le)
pub fn try_order_tm(state: &mut LuaState, a: &LuaValue, b: &LuaValue, event: TMS) -> Option<bool> {
    try_bin_tm(state, a, b, event).and_then(|v| v.as_bool())
}

/// Get type name for a LuaValue
pub fn obj_typename(val: &LuaValue) -> &'static str {
    match val {
        LuaValue::Nil => "nil",
        LuaValue::Bool(_) => "boolean",
        LuaValue::Int(_) | LuaValue::Float(_) => "number",
        LuaValue::Str(_) => "string",
        LuaValue::Table(_) => "table",
        LuaValue::Function(_) => "function",
        LuaValue::UserData(_) => "userdata",
        LuaValue::Thread(_) => "thread",
        LuaValue::Upvalue(_) => "upvalue",
        _ => "no value"
    }
}

/// Dynamic metamethod registry for extensibility
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref DYNAMIC_METAMETHODS: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
}

/// Register a new (custom) metamethod name, returning its dynamic index
pub fn register_metamethod(name: &str) -> usize {
    let mut reg = DYNAMIC_METAMETHODS.write().unwrap();
    let idx = reg.len() + TMS::COUNT;
    reg.entry(name.to_string()).or_insert(idx);
    idx
}

/// Lookup a dynamic metamethod index by name
pub fn get_dynamic_metamethod_index(name: &str) -> Option<usize> {
    DYNAMIC_METAMETHODS.read().unwrap().get(name).copied()
}

/// Lookup a metamethod (static or dynamic) in a table's metatable
pub fn get_any_tm(table: &LuaTable, name: &str) -> Option<LuaValue> {
    table.get_metatable().and_then(|mt| mt.get(&LuaValue::Str(name.to_string())))
}

/// Call any metamethod (static or dynamic)
pub fn call_any_tm(state: &mut LuaState, f: &LuaValue, args: &[LuaValue]) -> Option<LuaValue> {
    // In a real implementation, push args and call function in VM
    // Here, just a stub
    let _ = (state, f, args);
    None
}

/// VM integration: call a metamethod as a Lua function in the VM
pub fn call_tm_vm(state: &mut LuaState, f: &LuaValue, args: &[LuaValue]) -> Option<LuaValue> {
    // Example: push function and args, call in VM, pop result
    // This assumes LuaState has push, call_function, and pop methods
    state.push(f.clone());
    for arg in args {
        state.push(arg.clone());
    }
    // Call function with n arguments, expecting 1 result
    let nargs = args.len();
    let ok = state.call_function(nargs, 1); // returns true if call succeeded
    if ok {
        state.pop(1) // pop and return result
    } else {
        None
    }
}

/// VM integration: try a binary metamethod and return result (or fallback)
pub fn try_bin_tm_vm(state: &mut LuaState, a: &LuaValue, b: &LuaValue, event: TMS, fallback: impl Fn() -> Option<LuaValue>) -> Option<LuaValue> {
    let mt_a = a.get_metatable();
    let mt_b = b.get_metatable();
    let mm = mt_a.and_then(|mt| mt.get(&LuaValue::Str(event.name().to_string())))
        .or_else(|| mt_b.and_then(|mt| mt.get(&LuaValue::Str(event.name().to_string()))));
    if let Some(f) = mm {
        call_tm_vm(state, &f, &[a.clone(), b.clone()])
    } else {
        fallback()
    }
}

/// VM integration: try a custom metamethod by name and return result (or fallback)
pub fn try_custom_tm_vm(state: &mut LuaState, a: &LuaValue, b: &LuaValue, name: &str, fallback: impl Fn() -> Option<LuaValue>) -> Option<LuaValue> {
    let mt_a = a.get_metatable();
    let mt_b = b.get_metatable();
    let mm = mt_a.and_then(|mt| mt.get(&LuaValue::Str(name.to_string())))
        .or_else(|| mt_b.and_then(|mt| mt.get(&LuaValue::Str(name.to_string()))));
    if let Some(f) = mm {
        call_tm_vm(state, &f, &[a.clone(), b.clone()])
    } else {
        fallback()
    }
}

/// Example: Try a custom metamethod (by name)
pub fn try_custom_tm(state: &mut LuaState, a: &LuaValue, b: &LuaValue, name: &str) -> Option<LuaValue> {
    let mt_a = a.get_metatable();
    let mt_b = b.get_metatable();
    let mm = mt_a.and_then(|mt| mt.get(&LuaValue::Str(name.to_string())))
        .or_else(|| mt_b.and_then(|mt| mt.get(&LuaValue::Str(name.to_string()))));
    mm.and_then(|f| call_any_tm(state, &f, &[a.clone(), b.clone()]))
}

/// List all registered dynamic metamethods
pub fn list_dynamic_metamethods() -> Vec<String> {
    DYNAMIC_METAMETHODS.read().unwrap().keys().cloned().collect()
}

/// Remove a dynamic metamethod by name
pub fn unregister_metamethod(name: &str) -> bool {
    DYNAMIC_METAMETHODS.write().unwrap().remove(name).is_some()
}

/// Check if a metamethod (static or dynamic) exists for a value
pub fn has_any_tm(val: &LuaValue, name: &str) -> bool {
    val.get_metatable()
        .map_or(false, |mt| mt.contains_key(&LuaValue::Str(name.to_string())))
}

/// Get all metamethods (static and dynamic) for a value
pub fn all_metamethods(val: &LuaValue) -> Vec<String> {
    val.get_metatable()
        .map(|mt| mt.keys().filter_map(|k| k.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// Utility: pretty-print all registered dynamic metamethods
pub fn print_dynamic_metamethods() {
    let list = list_dynamic_metamethods();
    if list.is_empty() {
        println!("[ltm] No dynamic metamethods registered.");
    } else {
        println!("[ltm] Dynamic metamethods:");
        for name in list {
            println!("  - {}", name);
        }
    }
}

