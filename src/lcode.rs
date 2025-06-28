//! lcode.rs
//! Lua bytecode generator module in Rust.
//! Adapted from Lua 5.4 `lcode.c` for the Skyl project.

use std::os::raw::c_int;

use crate::lparser::{FuncState, expdesc};
use crate::lopcodes::{OpCode, Instruction};
use crate::lobject::{NO_JUMP};

/// Mark that the given list is empty (no jump).
pub const NO_JUMP: c_int = -1;

/// Returns current program counter (next instruction to be generated).
#[inline(always)]
pub fn getlabel(fs: &FuncState) -> c_int {
    fs.pc
}

/// Emit an instruction to the function prototype and increment pc.
pub fn code(fs: &mut FuncState, i: Instruction) -> c_int {
    let pc = fs.pc;
    fs.f.code.push(i);
    fs.pc += 1;
    pc
}

/// Emit an ABC-format instruction.
/// A, B, and C are operands (signed integers).
pub fn code_abc(fs: &mut FuncState, op: OpCode, a: c_int, b: c_int, c: c_int) -> c_int {
    let i = Instruction::encode_abc(op, a as u8, b as u8, c as u8);
    code(fs, i)
}

/// Emit an ABx-format instruction.
/// A and Bx operands (Bx usually a 18-bit constant or jump offset).
pub fn code_abx(fs: &mut FuncState, op: OpCode, a: c_int, bx: c_int) -> c_int {
    let i = Instruction::encode_abx(op, a as u8, bx as u32);
    code(fs, i)
}

/// Generate an unconditional jump instruction with placeholder offset.
pub fn jump(fs: &mut FuncState) -> c_int {
    // '0' is placeholder for jump offset, will be patched later
    code_abc(fs, OpCode::JMP, 0, 0, 0)
}

/// Patch a jump instruction at 'list' to jump to 'target'.
pub fn patchlist(fs: &mut FuncState, mut list: c_int, target: c_int) {
    while list != NO_JUMP {
        let next = get_jump(fs, list);
        patch_jump(fs, list, target);
        list = next;
    }
}

/// Patch all jumps in 'list' to jump to current position.
pub fn patchtohere(fs: &mut FuncState, list: c_int) {
    patchlist(fs, list, fs.pc);
}

/// Concatenate two jump lists, returning the head of the combined list.
pub fn concat(fs: &mut FuncState, list1: c_int, list2: c_int) -> c_int {
    if list2 == NO_JUMP {
        list1
    } else if list1 == NO_JUMP {
        list2
    } else {
        let mut list = list1;
        while get_jump(fs, list) != NO_JUMP {
            list = get_jump(fs, list);
        }
        set_jump(fs, list, list2);
        list1
    }
}

/// Returns the jump offset stored in instruction at 'pc'.
fn get_jump(fs: &FuncState, pc: c_int) -> c_int {
    let i = fs.f.code[pc as usize];
    Instruction::get_sbx(i)
}

/// Sets the jump offset for instruction at 'pc' to point to 'dest'.
fn patch_jump(fs: &mut FuncState, pc: c_int, dest: c_int) {
    let offset = dest - (pc + 1);
    let old_inst = fs.f.code[pc as usize];
    let new_inst = Instruction::set_sbx(old_inst, offset);
    fs.f.code[pc as usize] = new_inst;
}

/// Converts expression 'e' to a value stored in a register,
/// allocating a register if necessary.
pub fn exp2reg(fs: &mut FuncState, e: &mut expdesc, reg: c_int) {
    exp2anyreg(fs, e);
    if e.k != expdesc::VNONRELOC || e.info != reg {
        code_abc(fs, OpCode::MOVE, reg, e.info, 0);
        e.info = reg;
        e.k = expdesc::VRELOCABLE;
    }
}

/// Converts expression 'e' to any register, allocating if needed.
/// Returns the register index.
pub fn exp2anyreg(fs: &mut FuncState, e: &mut expdesc) -> c_int {
    match e.k {
        expdesc::VNONRELOC => e.info,
        expdesc::VRELOCABLE => {
            e.k = expdesc::VNONRELOC;
            e.info
        }
        expdesc::VCONST => {
            let r = luaK_exp2const(fs, e);
            e.info = r;
            e.k = expdesc::VNONRELOC;
            r
        }
        _ => {
            luaK_dischargevars(fs, e);
            e.info
        }
    }
}

/// Converts expression 'e' to a constant in the function's constant table.
/// Returns register holding the constant.
pub fn luaK_exp2const(fs: &mut FuncState, e: &mut expdesc) -> c_int {
    // Example for number constants
    match &e.k {
        expdesc::VKNUM => {
            let idx = addk(fs, e.nval);
            code_abx(fs, OpCode::LOADK, 0, idx)
        }
        // Add other constant types here...
        _ => unimplemented!("luaK_exp2const: unsupported constant type"),
    }
}

/// Discharges variables and relocatable expressions into registers.
pub fn luaK_dischargevars(fs: &mut FuncState, e: &mut expdesc) {
    match e.k {
        expdesc::VLOCAL | expdesc::VUPVAL | expdesc::VGLOBAL | expdesc::VINDEXED => {
            // Generate code to load variable value into a register
            // Implementation dependent on expression type
            unimplemented!()
        }
        _ => {}
    }
}

/// Emits an instruction to set a range of registers to nil.
pub fn luaK_nil(fs: &mut FuncState, from: c_int, n: c_int) {
    if n <= 0 {
        return;
    }
    code_abc(fs, OpCode::LOADNIL, from, n - 1, 0);
}

/// Moves expression to next free register.
pub fn luaK_exp2nextreg(fs: &mut FuncState, e: &mut expdesc) {
    luaK_exp2anyreg(fs, e);
    fs.freereg += 1;
}

/// Returns the index of the free register.
pub fn luaK_getfreereg(fs: &FuncState) -> c_int {
    fs.freereg
}

/// Allocates a new register.
pub fn luaK_reserveregs(fs: &mut FuncState, n: c_int) {
    fs.freereg += n;
}

/// Frees the registers allocated since last reserve.
pub fn luaK_freereg(fs: &mut FuncState, reg: c_int) {
    if reg + 1 == fs.freereg {
        fs.freereg -= 1;
    }
}

/// Returns true if expression is a constant (VKNUM, VKSTR, etc.)
pub fn luaK_isconstant(e: &expdesc) -> bool {
    matches!(e.k, expdesc::VKNUM | expdesc::VKSTR | expdesc::VTRUE | expdesc::VFALSE)
}

/// Jumps if expression is true.
pub fn luaK_goiftrue(fs: &mut FuncState, e: &mut expdesc) -> c_int {
    // Implementation of conditional jump if expression evaluates to true
    unimplemented!()
}

/// Jumps if expression is false.
pub fn luaK_goiffalse(fs: &mut FuncState, e: &mut expdesc) -> c_int {
    // Implementation of conditional jump if expression evaluates to false
    unimplemented!()
}
/// Adds a constant to the function's constant table and returns its index.
pub fn addk(fs: &mut FuncState, value: f64) -> c_int {
    let idx = fs.f.k.len() as c_int;
    fs.f.k.push(value);
    idx
}
/// Adds a string constant to the function's constant table and returns its index.
pub fn addk_string(fs: &mut FuncState, value: &str) -> c    _int {
    let idx = fs.f.k.len() as c_int;
    fs.f.k.push(value.to_string());
    idx
}
/// Adds a boolean constant to the function's constant table and returns its index.
pub fn addk_boolean(fs: &mut FuncState, value: bool) -> c_int   {
    let idx = fs.f.k.len() as c_int;
    fs.f.k.push(value);
    idx
}