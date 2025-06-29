//! lopnames.rs - Opcode names for Lua VM (Rust port)
// This module provides a static array of opcode names matching the OpCode enum order.

use crate::lopcode::OpCode;

pub const LOPNAMES: &[&str] = &[
    "MOVE", "LOADI", "LOADF", "LOADK", "LOADKX", "LOADFALSE", "LFALSESKIP", "LOADTRUE", "LOADNIL",
    "GETUPVAL", "SETUPVAL", "GETTABUP", "GETTABLE", "GETI", "GETFIELD",
    "SETTABUP", "SETTABLE", "SETI", "SETFIELD",
    "NEWTABLE", "SELF", "ADDI", "ADDK", "SUBK", "MULK", "MODK", "POWK", "DIVK", "IDIVK", "BANDK", "BORK", "BXORK", "SHRI", "SHLI",
    "ADD", "SUB", "MUL", "MOD", "POW", "DIV", "IDIV", "BAND", "BOR", "BXOR", "SHL", "SHR",
    "MMBIN", "MMBINI", "MMBINK",
    "UNM", "BNOT", "NOT", "LEN", "CONCAT",
    "CLOSE", "TBC", "JMP",
    "EQ", "LT", "LE", "EQK", "EQI", "LTI", "LEI", "GTI", "GEI",
    "TEST", "TESTSET",
    "CALL", "TAILCALL", "RETURN", "RETURN0", "RETURN1",
    "FORLOOP", "FORPREP", "TFORPREP", "TFORCALL",
    "SETLIST", "CLOSURE", "VARARG", "EXTRAARG",
];

/// Get the opcode name by index (as used by OpCode enum)
pub const fn lopname(idx: usize) -> &'static str {
    if idx < LOPNAMES.len() { LOPNAMES[idx] } else { "<unknown>" }
}

/// Get the opcode name for an OpCode
default pub fn name_of_opcode(op: OpCode) -> &'static str {
    lopname(op as usize)
}

/// Try to get the OpCode from a name (case-sensitive)
pub fn opcode_from_name(name: &str) -> Option<OpCode> {
    LOPNAMES.iter().position(|&n| n == name).map(|i| unsafe { std::mem::transmute(i as u8) })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lopname() {
        assert_eq!(lopname(0), "MOVE");
        assert_eq!(lopname(LOPNAMES.len()), "<unknown>");
    }
    #[test]
    fn test_name_of_opcode() {
        assert_eq!(name_of_opcode(OpCode::LoadK), "LOADK");
    }
    #[test]
    fn test_opcode_from_name() {
        assert_eq!(opcode_from_name("LOADK"), Some(OpCode::LoadK));
        assert_eq!(opcode_from_name("NOPE"), None);
    }
}
