//! lvm.rs
//! Lua Virtual Machine core interpreter module.
//! Executes Lua bytecode instructions.
//! Adapted and translated from Lua 5.4 `lvm.c`.

use std::os::raw::c_int;
use crate::lobject::{lua_State, TValue, lua_Number};
use crate::lopcodes::{Instruction, OpCode, GETARG_A, GETARG_B, GETARG_C, GETARG_Bx, GETARG_sBx};
use crate::lapi::{lua_pushnumber, lua_pushnil, lua_pop};
use crate::lfunc::{Proto, Closure};

/// The Lua VM main interpreter loop.
/// Executes bytecode instructions in `ci->func->p->code`.
pub unsafe fn luaV_execute(L: *mut lua_State) {
    let mut ci = (*L).ci;         // Call info for current function
    let mut cl = (*ci).func;      // Closure being executed
    let mut k: *const TValue;
    let mut base = (*ci).func.offset(1); // Base register of function stack frame
    let mut pc = (*ci).u.l.savedpc;

    // Shortcut references
    let mut instructions = (*(*cl).cl.p).code.as_ptr();

    // Main fetch-decode-execute loop
    loop {
        let instruction = *pc;
        pc = pc.offset(1);

        // Decode instruction opcode and args
        let op = OpCode::from_u8(instruction.get_opcode());
        let a = instruction.get_arg_a() as usize;
        let b = instruction.get_arg_b() as usize;
        let c = instruction.get_arg_c() as usize;
        let bx = instruction.get_arg_bx();
        let sbx = instruction.get_arg_sbx();

        match op {
            OpCode::MOVE => {
                // R(A) := R(B)
                let rb = base.offset(b as isize);
                let ra = base.offset(a as isize);
                *ra = *rb;
            }
            OpCode::LOADK => {
                // R(A) := Kst(Bx)
                k = (*(*cl).cl.p).k.as_ptr().offset(bx as isize);
                *base.offset(a as isize) = *k;
            }
            OpCode::LOADBOOL => {
                // R(A) := (Bool)B; if C != 0 skip next instruction
                *base.offset(a as isize) = TValue::from_bool(b != 0);
                if c != 0 {
                    pc = pc.offset(1);
                }
            }
            OpCode::LOADNIL => {
                // R(A) to R(A+B) := nil
                for i in 0..=b {
                    *base.offset((a + i) as isize) = TValue::nil();
                }
            }
            OpCode::GETUPVAL => {
                // R(A) := UpValue[B]
                let upval = (*cl).upvals[b].as_ref();
                *base.offset(a as isize) = *upval.val();
            }
            OpCode::GETGLOBAL => {
                // R(A) := Gbl[Kst(Bx)]
                let kname = (*(*cl).cl.p).k[bx as usize].to_string();
                let val = luaH_get(L, &(*L).l_env, &kname);
                *base.offset(a as isize) = val;
            }
            OpCode::SETGLOBAL => {
                // Gbl[Kst(Bx)] := R(A)
                let kname = (*(*cl).cl.p).k[bx as usize].to_string();
                luaH_set(L, &mut (*L).l_env, &kname, base.offset(a as isize));
            }
            OpCode::CALL => {
                // R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
                let n_args = b - 1;
                let n_results = c - 1;
                luaD_call(L, base.offset(a as isize), n_args, n_results);
                base = (*ci).func.offset(1);
            }
            OpCode::RETURN => {
                // return R(A), ... ,R(A+B-2)
                luaD_return(L, base.offset(a as isize), b - 1);
                return; // Return from this function frame
            }
            // Add other opcodes here with their implementations...

            _ => {
                panic!("Opcode {:?} not implemented yet!", op);
            }
        }
    }
}

/// Helper functions used inside VM:

/// Get a value from a Lua table (simplified)
unsafe fn luaH_get(L: *mut lua_State, table: *const TValue, key: &str) -> TValue {
    // Implement hash table lookup
    unimplemented!()
}

/// Set a value in a Lua table (simplified)
unsafe fn luaH_set(L: *mut lua_State, table: *mut TValue, key: &str, val: *const TValue) {
    // Implement hash table insertion or update
    unimplemented!()
}

/// Call a Lua function with n_args arguments and expect n_results results.
unsafe fn luaD_call(L: *mut lua_State, func: *mut TValue, n_args: usize, n_results: usize) {
    // Setup new call frame and execute function
    unimplemented!()
}

/// Return from a Lua function call.
unsafe fn luaD_return(L: *mut lua_State, first_result: *mut TValue, n_results: usize) {
    // Handle function return and stack cleanup
    unimplemented!()
}
use std::ptr;
use std::ffi::CString;

pub type lua_Number = f64;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum LuaType {
    Nil,
    Boolean,
    Number,
    String,
    Table,
    Function,
    // ... more types as needed
}

#[repr(C)]
pub struct TValue {
    pub tt: LuaType,
    pub value: TValueValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union TValueValue {
    pub b: bool,
    pub n: lua_Number,
    pub s: *const i8,
    pub p: *mut std::ffi::c_void, // generic pointer for tables/functions etc.
}

impl TValue {
    pub fn nil() -> Self {
        TValue {
            tt: LuaType::Nil,
            value: TValueValue { b: false },
        }
    }
    pub fn from_bool(b: bool) -> Self {
        TValue {
            tt: LuaType::Boolean,
            value: TValueValue { b },
        }
    }
    pub fn from_number(n: lua_Number) -> Self {
        TValue {
            tt: LuaType::Number,
            value: TValueValue { n },
        }
    }
    pub fn from_string(s: *const i8) -> Self {
        TValue {
            tt: LuaType::String,
            value: TValueValue { s },
        }
    }
}

// Lua function closure
#[repr(C)]
pub struct Closure {
    pub cl: ClosureType,
    pub upvals: *mut TValue,  // pointer to upvalues (simplified)
}

#[repr(C)]
pub union ClosureType {
    pub p: *mut Proto, // Lua closure proto
    // pub c: LuaCFunction, // native C closure, omitted for simplicity
}

#[repr(C)]
pub struct Proto {
    pub code: Vec<Instruction>,
    pub k: Vec<TValue>, // constants
    // ... other fields like debug info, upvalues, etc.
}

// Lua call frame
#[repr(C)]
pub struct CallInfo {
    pub func: *mut TValue,
    pub top: *mut TValue,
    pub u: CallInfoUnion,
}

#[repr(C)]
pub union CallInfoUnion {
    pub l: CallInfoL,
    // ... other variants
}

#[repr(C)]
pub struct CallInfoL {
    pub savedpc: *const Instruction,
}

#[repr(C)]
pub struct lua_State {
    pub ci: *mut CallInfo,
    pub top: *mut TValue,
    pub l_env: TValue,
    // ... other Lua VM state fields
}
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Instruction(pub u32);

impl Instruction {
    pub fn get_opcode(&self) -> u8 {
        (self.0 & 0x3F) as u8
    }

    pub fn get_arg_a(&self) -> u8 {
        ((self.0 >> 6) & 0xFF) as u8
    }

    pub fn get_arg_b(&self) -> u8 {
        ((self.0 >> 23) & 0x1FF) as u8
    }

    pub fn get_arg_c(&self) -> u8 {
        ((self.0 >> 14) & 0x1FF) as u8
    }

    pub fn get_arg_bx(&self) -> u32 {
        (self.0 >> 14) & 0x3FFFF
    }

    pub fn get_arg_sbx(&self) -> i32 {
        (self.get_arg_bx() as i32) - 131071 // bias for signed offset
    }

    pub fn encode_abc(opcode: OpCode, a: u8, b: u8, c: u8) -> Instruction {
        Instruction(
            (opcode as u32)
                | ((a as u32) << 6)
                | ((c as u32) << 14)
                | ((b as u32) << 23),
        )
    }

    pub fn encode_abx(opcode: OpCode, a: u8, bx: u32) -> Instruction {
        Instruction((opcode as u32) | ((a as u32) << 6) | (bx << 14))
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    MOVE = 0,
    LOADK = 1,
    LOADBOOL = 2,
    LOADNIL = 3,
    GETUPVAL = 4,
    GETGLOBAL = 5,
    SETGLOBAL = 6,
    CALL = 7,
    RETURN = 8,
    // ... add all Lua opcodes as needed
}

impl OpCode {
    pub fn from_u8(byte: u8) -> OpCode {
        match byte {
            0 => OpCode::MOVE,
            1 => OpCode::LOADK,
            2 => OpCode::LOADBOOL,
            3 => OpCode::LOADNIL,
            4 => OpCode::GETUPVAL,
            5 => OpCode::GETGLOBAL,
            6 => OpCode::SETGLOBAL,
            7 => OpCode::CALL,
            8 => OpCode::RETURN,
            _ => panic!("Unknown opcode {}", byte),
        }
    }
}

mod lmathlib;

use crate::lmathlib::luaopen_math;
use crate::lapi::{luaL_openlibs, luaL_requiref, lua_pop, lua_State};

pub unsafe fn luaL_openlibs(L: *mut lua_State) {
    // Open the standard Lua libraries

    // ... open other libs ...

    // Register math library
    luaL_requiref(L, cstr!("math"), Some(luaopen_math), 1);
    lua_pop(L, 1);

    // ... open other libs ...
}
