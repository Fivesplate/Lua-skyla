//! lopcode.rs - Lua VM opcodes, instruction formats, and helpers (Rust port)
// More compact, type-safe, and extensible than original lopcodes.c/h

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpCode {
    Move, LoadI, LoadF, LoadK, LoadKX, LoadFalse, LFalseSkip, LoadTrue, LoadNil,
    GetUpval, Setupval, GetTabUp, GetTable, GetI, GetField,
    SetTabUp, SetTable, SetI, SetField,
    NewTable, SelfOp, AddI, AddK, SubK, MulK, ModK, PowK, DivK, IDivK, BandK, BorK, BxorK, Shri, Shli,
    Add, Sub, Mul, Mod, Pow, Div, IDiv, Band, Bor, Bxor, Shl, Shr,
    MMBin, MMBinI, MMBinK,
    Unm, BNot, Not, Len, Concat,
    Close, TBC, Jmp,
    Eq, Lt, Le, EqK, EqI, LtI, LeI, GtI, GeI,
    Test, TestSet,
    Call, TailCall, Return, Return0, Return1,
    ForLoop, ForPrep, TForPrep, TForCall,
    SetList, Closure, VarArg, ExtraArg,
}

/// Instruction format (A, B, C, Ax, sBx, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpMode {
    ABC, ABx, AsBx, Ax, sJ, vABC,
}

/// Opcode metadata (mode, name, etc.)
pub struct OpCodeInfo {
    pub name: &'static str,
    pub mode: OpMode,
    pub has_arg_a: bool,
    pub has_arg_b: bool,
    pub has_arg_c: bool,
    pub is_mm: bool, // is metamethod
    pub test_flag: bool, // is test
}

/// Table of opcode metadata (indexed by OpCode as usize)
pub const OPCODE_INFOS: &[OpCodeInfo] = &[
    OpCodeInfo { name: "MOVE",      mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: true,  has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADI",     mode: OpMode::AsBx, has_arg_a: true,  has_arg_b: true,  has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADF",     mode: OpMode::AsBx, has_arg_a: true,  has_arg_b: true,  has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADK",     mode: OpMode::ABx,  has_arg_a: true,  has_arg_b: true,  has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADKX",    mode: OpMode::ABx,  has_arg_a: true,  has_arg_b: false, has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADFALSE", mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: false, has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LFALSESKIP",mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: false, has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADTRUE",  mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: false, has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "LOADNIL",   mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: false, has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "GETUPVAL",  mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: false, has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "SETUPVAL",  mode: OpMode::ABC,  has_arg_a: false, has_arg_b: true,  has_arg_c: false, is_mm: false, test_flag: false },
    OpCodeInfo { name: "GETTABUP",  mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "GETTABLE",  mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "GETI",      mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "GETFIELD",  mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "SETTABUP",  mode: OpMode::ABC,  has_arg_a: false, has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "SETTABLE",  mode: OpMode::ABC,  has_arg_a: false, has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "SETI",      mode: OpMode::ABC,  has_arg_a: false, has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "SETFIELD",  mode: OpMode::ABC,  has_arg_a: false, has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "NEWTABLE",  mode: OpMode::vABC, has_arg_a: true,  has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    OpCodeInfo { name: "SELF",      mode: OpMode::ABC,  has_arg_a: true,  has_arg_b: true,  has_arg_c: true,  is_mm: false, test_flag: false },
    // ...continue for all opcodes, matching the C order and metadata...
];

/// Instruction encoding/decoding helpers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction(pub u32);

impl Instruction {
    pub fn opcode(self) -> OpCode {
        // Assume opcode is in the lowest 6 bits
        unsafe { std::mem::transmute((self.0 & 0x3F) as u8) }
    }
    pub fn a(self) -> u8 { ((self.0 >> 6) & 0xFF) as u8 }
    pub fn b(self) -> u16 { ((self.0 >> 23) & 0x1FF) as u16 }
    pub fn c(self) -> u16 { ((self.0 >> 14) & 0x1FF) as u16 }
    pub fn bx(self) -> u32 { ((self.0 >> 14) & 0x3FFFF) as u32 }
    pub fn sbx(self) -> i32 { self.bx() as i32 - 131071 }
    pub fn ax(self) -> u32 { (self.0 >> 6) as u32 }
    // ...add more as needed...
}

/// Compact opcode name lookup
default impl OpCode {
    pub fn name(self) -> &'static str {
        OPCODE_INFOS[self as usize].name
    }
    pub fn mode(self) -> OpMode {
        OPCODE_INFOS[self as usize].mode
    }
}

/// Macro for compact opcode info definition
macro_rules! opinfo {
    ($name:expr, $mode:expr, $a:expr, $b:expr, $c:expr, $mm:expr, $test:expr) => {
        OpCodeInfo { name: $name, mode: $mode, has_arg_a: $a, has_arg_b: $b, has_arg_c: $c, is_mm: $mm, test_flag: $test }
    };
}

/// Fast static lookup for opcode properties
impl OpCode {
    pub fn is_metamethod(self) -> bool {
        OPCODE_INFOS[self as usize].is_mm
    }
    pub fn is_test(self) -> bool {
        OPCODE_INFOS[self as usize].test_flag
    }
    pub fn has_a(self) -> bool {
        OPCODE_INFOS[self as usize].has_arg_a
    }
    pub fn has_b(self) -> bool {
        OPCODE_INFOS[self as usize].has_arg_b
    }
    pub fn has_c(self) -> bool {
        OPCODE_INFOS[self as usize].has_arg_c
    }
}

/// Example: get all opcodes with a given property
pub fn opcodes_with<F: Fn(&OpCodeInfo) -> bool>(pred: F) -> Vec<OpCode> {
    (0..OPCODE_INFOS.len())
        .filter(|&i| pred(&OPCODE_INFOS[i]))
        .map(|i| unsafe { std::mem::transmute(i as u8) })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_opcode_name() {
        assert_eq!(OpCode::Move.name(), "MOVE");
    }
    #[test]
    fn test_instruction_fields() {
        let instr = Instruction(0b000001_00000010_00000011_00000000_00000000);
        assert_eq!(instr.opcode(), OpCode::Move);
        assert_eq!(instr.a(), 2);
    }
    #[test]
    fn test_opcode_properties() {
        assert!(OpCode::Move.has_a());
        assert!(!OpCode::Move.is_metamethod());
    }
    #[test]
    fn test_opcodes_with() {
        let mm_ops = opcodes_with(|info| info.is_mm);
        assert!(mm_ops.is_empty() || mm_ops.iter().all(|op| op.is_metamethod()));
    }
}
