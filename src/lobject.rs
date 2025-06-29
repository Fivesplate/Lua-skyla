//! lobject.rs - Core Lua object utilities and arithmetic (Rust port)
// Ported from lobject.c

use crate::lstate::lua_State;
use crate::llimits::*;
use crate::lmem::*;
use crate::lstring::*;
use crate::lvm::*;
use crate::ldebug::*;
use crate::ldo::*;
use std::cmp;
use std::f64;

/// Computes ceil(log2(x))
pub fn luaO_ceillog2(mut x: u32) -> u8 {
    const LOG_2: [u8; 256] = [
        0,1,2,2,3,3,3,3,4,4,4,4,4,4,4,4,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,
        6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,
        7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
        7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
        8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
        8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
        8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,
        8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8
    ];
    let mut l = 0;
    x -= 1;
    while x >= 256 { l += 8; x >>= 8; }
    (l + LOG_2[x as usize]) as u8
}

/*
** Encodes 'p'% as a floating-point byte, represented as (eeeexxxx).
** The exponent is represented using excess-7. Mimicking IEEE 754, the
** representation normalizes the number when possible, assuming an extra
** 1 before the mantissa (xxxx) and adding one to the exponent (eeee)
** to signal that. So, the real value is (1xxxx) * 2^(eeee - 7 - 1) if
** eeee != 0, and (xxxx) * 2^-7 otherwise (subnormal numbers).
*/
pub fn luaO_codeparam(mut p: u32) -> u8 {
    if p >= ((0x1F as u64) << (0xF - 7 - 1)) as u32 * 100 {
        0xFF
    } else {
        p = (p * 128 + 99) / 100;
        if p < 0x10 {
            p as u8
        } else {
            let log = luaO_ceillog2(p + 1) - 5;
            (((p >> log) - 0x10) | ((log + 1) << 4)) as u8
        }
    }
}

/// Applies a floating-point byte parameter to an integer
pub fn luaO_applyparam(p: u8, x: i64) -> i64 {
    let mut m = (p & 0xF) as i64;
    let mut e = (p >> 4) as i32;
    if e > 0 {
        e -= 1;
        m += 0x10;
    }
    e -= 7;
    if e >= 0 {
        if x < (MAX_LMEM / 0x1F) >> e {
            (x * m) << e
        } else {
            MAX_LMEM
        }
    } else {
        let e = -e;
        if x < MAX_LMEM / 0x1F {
            (x * m) >> e
        } else if (x >> e) < MAX_LMEM / 0x1F {
            (x >> e) * m
        } else {
            MAX_LMEM
        }
    }
}

/// Hexadecimal value for a character (0-9, a-f, A-F)
pub fn luaO_hexavalue(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

/// Convert a string to an integer (supports decimal and hex)
pub fn luaO_str2int(s: &str) -> Option<i64> {
    let s = s.trim();
    let (neg, s) = match s.chars().next() {
        Some('-') => (true, &s[1..]),
        Some('+') => (false, &s[1..]),
        _ => (false, s),
    };
    let s = s.trim_start();
    if s.starts_with("0x") || s.starts_with("0X") {
        i64::from_str_radix(&s[2..], 16).ok().map(|v| if neg { -v } else { v })
    } else {
        s.parse::<i64>().ok().map(|v| if neg { -v } else { v })
    }
}

/// Convert a string to a float (locale-independent, basic)
pub fn luaO_str2num(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

/// Convert a number to a string (integer or float)
pub fn luaO_num2str(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else {
        format!("{}", n)
    }
}

/// Convert a number to a string, adding ".0" if it looks like an integer
pub fn luaO_num2str_dot(n: f64) -> String {
    let s = luaO_num2str(n);
    if s.find('.').is_none() && s.find('e').is_none() && s.find('E').is_none() {
        format!("{}.0", s)
    } else {
        s
    }
}

/// UTF-8 escape for a Unicode codepoint
pub fn luaO_utf8esc(x: u32) -> Vec<u8> {
    let mut buf = [0u8; 4];
    let n = char::from_u32(x).unwrap_or('\u{FFFD}').encode_utf8(&mut buf).len();
    buf[..n].to_vec()
}

/// Format a chunk id for error messages (like luaO_chunkid)
pub fn luaO_chunkid(source: &str, bufflen: usize) -> String {
    const RETS: &str = "...";
    const PRE: &str = "[string \"";
    const POS: &str = "]";
    if let Some(rest) = source.strip_prefix('=') {
        if rest.len() <= bufflen {
            rest.to_string()
        } else {
            let mut out = String::with_capacity(bufflen);
            out.push_str(&rest[..bufflen.saturating_sub(1)]);
            out
        }
    } else if let Some(rest) = source.strip_prefix('@') {
        if rest.len() <= bufflen {
            rest.to_string()
        } else {
            let mut out = String::with_capacity(bufflen);
            out.push_str(RETS);
            let keep = bufflen.saturating_sub(RETS.len());
            if rest.len() > keep {
                out.push_str(&rest[rest.len() - keep..]);
            } else {
                out.push_str(rest);
            }
            out
        }
    } else {
        // string; format as [string "source"]
        let mut out = String::with_capacity(bufflen);
        out.push_str(PRE);
        let mut srclen = source.len();
        let nl = source.find('\n');
        let mut bufflen = bufflen.saturating_sub(PRE.len() + RETS.len() + POS.len() + 1);
        if let Some(nl) = nl {
            srclen = nl;
        }
        if srclen < bufflen {
            out.push_str(&source[..srclen]);
        } else {
            out.push_str(&source[..bufflen]);
            out.push_str(RETS);
        }
        out.push_str(POS);
        out
    }
}

/// Arithmetic operations for Lua values (integer and float)
pub fn luaO_add(a: f64, b: f64) -> f64 { a + b }
pub fn luaO_sub(a: f64, b: f64) -> f64 { a - b }
pub fn luaO_mul(a: f64, b: f64) -> f64 { a * b }
pub fn luaO_div(a: f64, b: f64) -> f64 { a / b }
pub fn luaO_mod(a: f64, b: f64) -> f64 { a % b }
pub fn luaO_pow(a: f64, b: f64) -> f64 { a.powf(b) }
pub fn luaO_unm(a: f64) -> f64 { -a }

/// Integer bitwise operations
pub fn luaO_band(a: i64, b: i64) -> i64 { a & b }
pub fn luaO_bor(a: i64, b: i64) -> i64 { a | b }
pub fn luaO_bxor(a: i64, b: i64) -> i64 { a ^ b }
pub fn luaO_bnot(a: i64) -> i64 { !a }
pub fn luaO_shl(a: i64, b: u32) -> i64 { a << b }
pub fn luaO_shr(a: i64, b: u32) -> i64 { a >> b }

/// Equality and comparison helpers
pub fn luaO_eqnum(a: f64, b: f64) -> bool { (a - b).abs() < f64::EPSILON }
pub fn luaO_eqint(a: i64, b: i64) -> bool { a == b }
pub fn luaO_lt(a: f64, b: f64) -> bool { a < b }
pub fn luaO_le(a: f64, b: f64) -> bool { a <= b }

/// Set a node's key as 'dead' (used in Lua tables for deleted keys)
#[inline(always)]
pub fn setdeadkey<T>(node: &mut Node<T>) {
    node.key_is_dead = true;
}

/// Node structure for Lua tables (simplified)
pub struct Node<T> {
    pub key: Option<T>,
    pub value: Option<T>,
    pub key_is_dead: bool,
}

impl<T> Node<T> {
    pub fn new(key: Option<T>, value: Option<T>) -> Self {
        Node { key, value, key_is_dead: false }
    }
}

/// lmod: Lua module metadata (for module system, e.g. package.loaded)
pub struct LMod {
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub doc: Option<String>,
}

impl LMod {
    pub fn new(name: &str) -> Self {
        LMod {
            name: name.to_string(),
            version: None,
            author: None,
            doc: None,
        }
    }
}

/// Macro-like attribute for marking Lua internal functions (for inlining, visibility, etc.)
#[macro_export]
macro_rules! luai_func {
    // Usage: luai_func!(pub fn foo(args) -> Ret { ... })
    ($vis:vis fn $name:ident $args:tt $body:block) => {
        #[inline(always)]
        $vis fn $name $args $body
    };
}

// Example usage of luai_func macro for a Lua internal function
luai_func!(pub fn luaO_example_func(x: i32) -> i32 {
    x * 2
});

// --- Complex Lua object helpers and interop ---

/// A trait for Lua value types (for dynamic dispatch, type tags, etc.)
pub trait LuaValue: std::fmt::Debug + Send + Sync {
    fn type_name(&self) -> &'static str;
    fn as_number(&self) -> Option<f64> { None }
    fn as_integer(&self) -> Option<i64> { None }
    fn as_str(&self) -> Option<&str> { None }
    fn is_nil(&self) -> bool { false }
    fn is_truthy(&self) -> bool { true }
}

/// Example Lua value enum (expand as needed)
#[derive(Debug, Clone)]
pub enum LObject {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Table, // Placeholder for table type
    Function, // Placeholder for function type
    UserData, // Placeholder for user data
    // ... add more as needed ...
}

impl LuaValue for LObject {
    fn type_name(&self) -> &'static str {
        match self {
            LObject::Nil => "nil",
            LObject::Boolean(_) => "boolean",
            LObject::Integer(_) => "integer",
            LObject::Number(_) => "number",
            LObject::String(_) => "string",
            LObject::Table => "table",
            LObject::Function => "function",
            LObject::UserData => "userdata",
        }
    }
    fn as_number(&self) -> Option<f64> {
        match self {
            LObject::Number(n) => Some(*n),
            LObject::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    fn as_integer(&self) -> Option<i64> {
        match self {
            LObject::Integer(i) => Some(*i),
            LObject::Number(n) => Some(*n as i64),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            LObject::String(s) => Some(s),
            _ => None,
        }
    }
    fn is_nil(&self) -> bool {
        matches!(self, LObject::Nil)
    }
    fn is_truthy(&self) -> bool {
        !matches!(self, LObject::Nil | LObject::Boolean(false))
    }
}

/// Convert a Rust value to an LObject (for FFI or scripting interop)
pub fn to_lobject<T: Into<LObject>>(v: T) -> LObject {
    v.into()
}

/// Example: Implement From for common Rust types
impl From<i64> for LObject {
    fn from(i: i64) -> Self { LObject::Integer(i) }
}
impl From<f64> for LObject {
    fn from(n: f64) -> Self { LObject::Number(n) }
}
impl From<&str> for LObject {
    fn from(s: &str) -> Self { LObject::String(s.to_string()) }
}
impl From<String> for LObject {
    fn from(s: String) -> Self { LObject::String(s) }
}
impl From<bool> for LObject {
    fn from(b: bool) -> Self { LObject::Boolean(b) }
}

/// Example: Convert LObject to Rust types (if possible)
pub fn lobject_to_i64(obj: &LObject) -> Option<i64> { obj.as_integer() }
pub fn lobject_to_f64(obj: &LObject) -> Option<f64> { obj.as_number() }
pub fn lobject_to_str(obj: &LObject) -> Option<&str> { obj.as_str() }

/// Example: Table node with LObject keys/values
pub type LNode = Node<LObject>;

/// Mark a node as dead (for table GC)
pub fn lnode_setdeadkey(node: &mut LNode) { setdeadkey(node); }

/// Example: Create a Lua table node
pub fn lnode_new(key: LObject, value: LObject) -> LNode {
    Node::new(Some(key), Some(value))
}

/// Example: Module metadata for D/Rust interop
pub fn lmod_with_meta(name: &str, version: &str, author: &str, doc: &str) -> LMod {
    let mut m = LMod::new(name);
    m.version = Some(version.to_string());
    m.author = Some(author.to_string());
    m.doc = Some(doc.to_string());
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ceillog2() {
        assert_eq!(luaO_ceillog2(1), 0);
        assert_eq!(luaO_ceillog2(2), 1);
        assert_eq!(luaO_ceillog2(3), 2);
        assert_eq!(luaO_ceillog2(16), 4);
    }
    #[test]
    fn test_hexavalue() {
        assert_eq!(luaO_hexavalue(b'0'), 0);
        assert_eq!(luaO_hexavalue(b'9'), 9);
        assert_eq!(luaO_hexavalue(b'a'), 10);
        assert_eq!(luaO_hexavalue(b'F'), 15);
    }
    #[test]
    fn test_str2int() {
        assert_eq!(luaO_str2int("42"), Some(42));
        assert_eq!(luaO_str2int("-42"), Some(-42));
        assert_eq!(luaO_str2int("0x10"), Some(16));
    }
    #[test]
    fn test_str2num() {
        assert_eq!(luaO_str2num("3.14"), Some(3.14));
        assert_eq!(luaO_str2num("-2.5"), Some(-2.5));
    }
    #[test]
    fn test_num2str() {
        assert_eq!(luaO_num2str(42.0), "42");
        assert_eq!(luaO_num2str(3.14), "3.14");
    }
    #[test]
    fn test_utf8esc() {
        assert_eq!(luaO_utf8esc(0x41), vec![0x41]);
        assert_eq!(luaO_utf8esc(0x20AC), vec![0xE2, 0x82, 0xAC]);
    }
}

#[cfg(test)]
mod chunkid_tests {
    use super::*;
    #[test]
    fn test_chunkid_literal() {
        let s = luaO_chunkid("=foo", 10);
        assert_eq!(s, "foo");
    }
    #[test]
    fn test_chunkid_file() {
        let s = luaO_chunkid("@/very/long/path/to/file.lua", 10);
        assert!(s.starts_with("..."));
    }
    #[test]
    fn test_chunkid_string() {
        let s = luaO_chunkid("print('hi')", 20);
        assert!(s.starts_with("[string "));
    }
}

#[cfg(test)]
mod arith_tests {
    use super::*;
    #[test]
    fn test_add() { assert_eq!(luaO_add(2.0, 3.0), 5.0); }
    #[test]
    fn test_sub() { assert_eq!(luaO_sub(5.0, 3.0), 2.0); }
    #[test]
    fn test_mul() { assert_eq!(luaO_mul(2.0, 3.0), 6.0); }
    #[test]
    fn test_div() { assert_eq!(luaO_div(6.0, 3.0), 2.0); }
    #[test]
    fn test_mod() { assert_eq!(luaO_mod(7.0, 3.0), 1.0); }
    #[test]
    fn test_pow() { assert_eq!(luaO_pow(2.0, 3.0), 8.0); }
    #[test]
    fn test_unm() { assert_eq!(luaO_unm(2.0), -2.0); }
    #[test]
    fn test_band() { assert_eq!(luaO_band(6, 3), 2); }
    #[test]
    fn test_bor() { assert_eq!(luaO_bor(6, 3), 7); }
    #[test]
    fn test_bxor() { assert_eq!(luaO_bxor(6, 3), 5); }
    #[test]
    fn test_bnot() { assert_eq!(luaO_bnot(6), !6); }
    #[test]
    fn test_shl() { assert_eq!(luaO_shl(2, 2), 8); }
    #[test]
    fn test_shr() { assert_eq!(luaO_shr(8, 2), 2); }
    #[test]
    fn test_eqnum() { assert!(luaO_eqnum(1.0, 1.0)); }
    #[test]
    fn test_eqint() { assert!(luaO_eqint(42, 42)); }
    #[test]
    fn test_lt() { assert!(luaO_lt(1.0, 2.0)); }
    #[test]
    fn test_le() { assert!(luaO_le(2.0, 2.0)); }
}

#[cfg(test)]
mod luai_func_tests {
    use super::*;
    #[test]
    fn test_luai_func_macro() {
        assert_eq!(luaO_example_func(21), 42);
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;
    #[test]
    fn test_setdeadkey() {
        let mut node = Node::new(Some(1), Some(2));
        assert!(!node.key_is_dead);
        setdeadkey(&mut node);
        assert!(node.key_is_dead);
    }
    #[test]
    fn test_lmod() {
        let m = LMod::new("foo");
        assert_eq!(m.name, "foo");
        assert!(m.version.is_none());
    }
}

#[cfg(test)]
mod lobject_ext_tests {
    use super::*;
    #[test]
    fn test_lobject_type_name() {
        assert_eq!(LObject::Nil.type_name(), "nil");
        assert_eq!(LObject::Integer(1).type_name(), "integer");
        assert_eq!(LObject::String("foo".into()).type_name(), "string");
    }
    #[test]
    fn test_lobject_truthy() {
        assert!(!LObject::Nil.is_truthy());
        assert!(!LObject::Boolean(false).is_truthy());
        assert!(LObject::Boolean(true).is_truthy());
        assert!(LObject::Integer(0).is_truthy());
    }
    #[test]
    fn test_lobject_from() {
        let i: LObject = 42i64.into();
        let n: LObject = 3.14f64.into();
        let s: LObject = "bar".into();
        assert_eq!(i.as_integer(), Some(42));
        assert_eq!(n.as_number(), Some(3.14));
        assert_eq!(s.as_str(), Some("bar"));
    }
    #[test]
    fn test_lnode() {
        let mut node = lnode_new(LObject::Integer(1), LObject::String("v".into()));
        assert!(!node.key_is_dead);
        lnode_setdeadkey(&mut node);
        assert!(node.key_is_dead);
    }
    #[test]
    fn test_lmod_with_meta() {
        let m = lmod_with_meta("foo", "1.0", "me", "doc");
        assert_eq!(m.version.as_deref(), Some("1.0"));
        assert_eq!(m.author.as_deref(), Some("me"));
        assert_eq!(m.doc.as_deref(), Some("doc"));
    }
}
