//! lstrlib.rs - Standard library for string operations and pattern-matching (Rust port)
// Ported from lstrlib.c, with optional D-based helpers via FFI

// --- Rust module imports for C header equivalents ---
// Standard library equivalents
use std::f64; // for float limits
use std::cmp; // for min/max, etc.
use std::ptr;
use std::ffi;
use std::str;
use std::fmt;
use std::mem;
use std::os;
use std::env;
use std::collections::HashSet;

// Local Lua VM modules (assume these exist or will be created)
mod lua;
mod lauxlib;
mod lualib;
mod llimits;

/// Returns the length of the string
pub fn str_len(s: &str) -> usize {
    s.chars().count()
}

/// Returns a substring from start to end (1-based, inclusive)
pub fn str_sub(s: &str, start: isize, end: Option<isize>) -> String {
    let len = s.chars().count() as isize;
    let start = if start > 0 { start - 1 } else { len + start };
    let end = end.unwrap_or(-1);
    let end = if end >= 0 { end } else { len + end + 1 };
    s.chars().skip(start.max(0) as usize).take((end - start).max(0) as usize).collect()
}

/// Returns the string reversed
pub fn str_reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Returns the string in lowercase
pub fn str_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Returns the string in uppercase
pub fn str_upper(s: &str) -> String {
    s.to_uppercase()
}

/// Repeats the string n times, with optional separator
pub fn str_rep(s: &str, n: usize, sep: Option<&str>) -> String {
    if n == 0 { return String::new(); }
    let sep = sep.unwrap_or("");
    std::iter::repeat(s).take(n).collect::<Vec<_>>().join(sep)
}

/// Returns the bytes at the given positions (1-based)
pub fn str_byte(s: &str, start: isize, end: Option<isize>) -> Vec<u8> {
    let bytes = s.as_bytes();
    let len = bytes.len() as isize;
    let start = if start > 0 { start - 1 } else { len + start };
    let end = end.unwrap_or(start + 1);
    let end = if end >= 0 { end } else { len + end + 1 };
    bytes.iter().skip(start.max(0) as usize).take((end - start).max(0) as usize).copied().collect()
}

/// Returns a string from the given bytes
pub fn str_char(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

// --- Minimal Lua pattern-matching engine (partial, extensible) ---
use std::collections::HashSet;

/// Checks if a character matches a Lua pattern class (e.g., %a, %d, etc.)
fn match_class(c: char, class: char) -> bool {
    match class {
        'a' => c.is_ascii_alphabetic(),
        'd' => c.is_ascii_digit(),
        'l' => c.is_ascii_lowercase(),
        'u' => c.is_ascii_uppercase(),
        'w' => c.is_ascii_alphanumeric(),
        's' => c.is_ascii_whitespace(),
        'p' => c.is_ascii_punctuation(),
        'c' => c.is_ascii_control(),
        'x' => c.is_ascii_hexdigit(),
        'z' => c == '\0',
        'A' => !c.is_ascii_alphabetic(),
        'D' => !c.is_ascii_digit(),
        'L' => !c.is_ascii_lowercase(),
        'U' => !c.is_ascii_uppercase(),
        'W' => !c.is_ascii_alphanumeric(),
        'S' => !c.is_ascii_whitespace(),
        'P' => !c.is_ascii_punctuation(),
        'C' => !c.is_ascii_control(),
        'X' => !c.is_ascii_hexdigit(),
        'Z' => c != '\0',
        _ => c == class,
    }
}

/// Matches a single pattern item (char, class, or .)
fn match_one(c: char, pat: &mut std::str::Chars) -> bool {
    match pat.next() {
        Some('.') => true,
        Some('%') => {
            if let Some(class) = pat.next() {
                match_class(c, class)
            } else {
                false
            }
        }
        Some(ch) => c == ch,
        None => false,
    }
}

/// Minimal recursive pattern matcher (no captures, no balanced, no frontier)
fn match_lua_pat(s: &str, pat: &str) -> Option<(usize, usize)> {
    let s_chars: Vec<_> = s.chars().collect();
    let pat_chars: Vec<_> = pat.chars().collect();
    for i in 0..=s_chars.len() {
        if let Some(len) = match_here(&s_chars[i..], &pat_chars) {
            return Some((i + 1, i + len)); // 1-based
        }
    }
    None
}

fn match_here(s: &[char], pat: &[char]) -> Option<usize> {
    if pat.is_empty() {
        return Some(0);
    }
    let mut pat_iter = pat.iter().peekable();
    let mut s_idx = 0;
    while let Some(&&p) = pat_iter.peek() {
        if let Some(&&next) = pat_iter.clone().nth(1) {
            match next {
                '*' => {
                    pat_iter.next(); pat_iter.next();
                    let mut max = s_idx;
                    while s_idx < s.len() && match_pat_char(s[s_idx], p) {
                        s_idx += 1;
                    }
                    for j in (0..=s_idx).rev() {
                        if let Some(rest) = match_here(&s[j..], pat_iter.clone().collect::<Vec<_>>().as_slice()) {
                            return Some(j + rest);
                        }
                    }
                    return None;
                }
                '+' => {
                    pat_iter.next(); pat_iter.next();
                    if s_idx < s.len() && match_pat_char(s[s_idx], p) {
                        s_idx += 1;
                        while s_idx < s.len() && match_pat_char(s[s_idx], p) {
                            s_idx += 1;
                        }
                        for j in (1..=s_idx).rev() {
                            if let Some(rest) = match_here(&s[j..], pat_iter.clone().collect::<Vec<_>>().as_slice()) {
                                return Some(j + rest);
                            }
                        }
                    }
                    return None;
                }
                '?' => {
                    pat_iter.next(); pat_iter.next();
                    if s_idx < s.len() && match_pat_char(s[s_idx], p) {
                        if let Some(rest) = match_here(&s[s_idx + 1..], pat_iter.clone().collect::<Vec<_>>().as_slice()) {
                            return Some(1 + rest);
                        }
                    }
                    if let Some(rest) = match_here(&s[s_idx..], pat_iter.clone().collect::<Vec<_>>().as_slice()) {
                        return Some(rest);
                    }
                    return None;
                }
                _ => {}
            }
        }
        // Single char match
        pat_iter.next();
        if s_idx < s.len() && match_pat_char(s[s_idx], p) {
            s_idx += 1;
        } else {
            return None;
        }
    }
    Some(s_idx)
}

fn match_pat_char(c: char, p: char) -> bool {
    if p == '.' {
        true
    } else if p == '%' {
        false // handled in full engine
    } else {
        c == p
    }
}

/// Matches a character against a bracketed class (e.g., [abc], [^abc], [a-z])
fn match_bracket_class(c: char, pat: &[char]) -> Option<(bool, usize)> {
    if pat.is_empty() || pat[0] != '[' {
        return None;
    }
    let mut negate = false;
    let mut i = 1;
    if i < pat.len() && pat[i] == '^' {
        negate = true;
        i += 1;
    }
    let mut matched = false;
    while i < pat.len() && pat[i] != ']' {
        if i + 2 < pat.len() && pat[i + 1] == '-' && pat[i + 2] != ']' {
            // Range
            let start = pat[i];
            let end = pat[i + 2];
            if start <= c && c <= end {
                matched = true;
            }
            i += 3;
        } else {
            if pat[i] == c {
                matched = true;
            }
            i += 1;
        }
    }
    let consumed = i + 1; // include closing ]
    Some(((matched ^ negate), consumed))
}

/// Enhanced pattern matcher with bracket class and basic captures (returns captures)
fn match_lua_pat_captures(s: &str, pat: &str) -> Option<(usize, usize, Vec<String>)> {
    let s_chars: Vec<_> = s.chars().collect();
    let pat_chars: Vec<_> = pat.chars().collect();
    for i in 0..=s_chars.len() {
        if let Some((len, caps)) = match_here_captures(&s_chars[i..], &pat_chars, &mut Vec::new()) {
            return Some((i + 1, i + len, caps));
        }
    }
    None
}

fn match_here_captures(s: &[char], pat: &[char], caps: &mut Vec<String>) -> Option<(usize, Vec<String>)> {
    if pat.is_empty() {
        return Some((0, caps.clone()));
    }
    let mut pat_iter = 0;
    let mut s_idx = 0;
    let mut local_caps = caps.clone();
    while pat_iter < pat.len() {
        // Handle captures: ( ... )
        if pat[pat_iter] == '(' {
            let cap_start = s_idx;
            pat_iter += 1;
            let mut cap_pat = Vec::new();
            let mut depth = 1;
            while pat_iter < pat.len() && depth > 0 {
                if pat[pat_iter] == '(' { depth += 1; }
                if pat[pat_iter] == ')' { depth -= 1; }
                if depth > 0 { cap_pat.push(pat[pat_iter]); }
                pat_iter += 1;
            }
            if let Some((cap_len, mut sub_caps)) = match_here_captures(&s[s_idx..], &cap_pat, &mut Vec::new()) {
                let cap_str: String = s[s_idx..s_idx+cap_len].iter().collect();
                local_caps.push(cap_str);
                s_idx += cap_len;
                local_caps.append(&mut sub_caps);
            } else {
                return None;
            }
            continue;
        }
        // Bracket class
        if pat[pat_iter] == '[' {
            if let Some((matched, consumed)) = match_bracket_class(s.get(s_idx).copied().unwrap_or('\0'), &pat[pat_iter..]) {
                if matched {
                    s_idx += 1;
                    pat_iter += consumed;
                    continue;
                } else {
                    return None;
                }
            }
        }
        // Char class
        if pat[pat_iter] == '%' && pat_iter + 1 < pat.len() {
            if s_idx < s.len() && match_class(s[s_idx], pat[pat_iter + 1]) {
                s_idx += 1;
                pat_iter += 2;
                continue;
            } else {
                return None;
            }
        }
        // Dot
        if pat[pat_iter] == '.' {
            if s_idx < s.len() {
                s_idx += 1;
                pat_iter += 1;
                continue;
            } else {
                return None;
            }
        }
        // Literal
        if s_idx < s.len() && pat[pat_iter] == s[s_idx] {
            s_idx += 1;
            pat_iter += 1;
            continue;
        } else {
            return None;
        }
    }
    Some((s_idx, local_caps))
}

/// Returns all captures for the first match of a pattern
pub fn str_captures(s: &str, pat: &str) -> Vec<String> {
    if let Some((_start, _end, caps)) = match_lua_pat_captures(s, pat) {
        caps
    } else {
        Vec::new()
    }
}

/// Checks for Lua frontier pattern (%f[])
fn match_frontier(s: &[char], pos: usize, set: &[char]) -> bool {
    let prev = if pos == 0 { '\0' } else { s[pos - 1] };
    let curr = if pos < s.len() { s[pos] } else { '\0' };
    let in_set = |c| set.contains(&c);
    !in_set(prev) && in_set(curr)
}

/// Substitute captures in replacement string (e.g., %1, %2)
pub fn str_gsub_captures(s: &str, pat: &str, repl: &str) -> String {
    let mut out = String::new();
    let mut last = 0;
    let mut rest = s;
    let mut offset = 0;
    while let Some((start, end, caps)) = match_lua_pat_captures(rest, pat) {
        let start0 = start - 1;
        let end0 = end;
        out.push_str(&rest[..start0]);
        let mut rep = String::new();
        let mut chars = repl.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(nc) = chars.peek() {
                    if nc.is_ascii_digit() {
                        let idx = nc.to_digit(10).unwrap() as usize - 1;
                        if idx < caps.len() {
                            rep.push_str(&caps[idx]);
                        }
                        chars.next();
                        continue;
                    }
                }
            }
            rep.push(c);
        }
        out.push_str(&rep);
        rest = &rest[end0..];
        offset += end0;
    }
    out.push_str(rest);
    out
}

// --- Extended quantifier support for bracket/capture ---
// (This is a stub for demonstration; a full engine would require a full parser)
// For now, bracket/capture quantifiers are handled as single matches.

// --- Tests for advanced pattern features ---
#[cfg(test)]
mod advanced_pattern_tests {
    use super::*;
    #[test]
    fn test_bracket_class() {
        assert!(str_match("abc", "[ab]c"));
        assert!(str_match("xbc", "[a-z]bc"));
        assert!(!str_match("1bc", "[a-z]bc"));
        assert!(str_match("1bc", "[^a-z]bc"));
    }
    #[test]
    fn test_captures() {
        let caps = str_captures("foo123bar", "foo(%d+)(%a+)");
        assert_eq!(caps, vec!["123", "bar"]);
    }
    #[test]
    fn test_gsub_captures() {
        let s = "foo123bar foo456baz";
        let out = str_gsub_captures(s, "foo(%d+)(%a+)", "bar-%2-%1");
        assert_eq!(out, "bar-bar-123 bar-baz-456");
    }
}

// --- Tests for pattern engine ---
#[cfg(test)]
mod pattern_tests {
    use super::*;
    #[test]
    fn test_dot() {
        assert!(str_match("abc", ".b."));
        assert!(!str_match("abc", ".d."));
    }
    #[test]
    fn test_star() {
        assert!(str_match("aaab", "a*b"));
        assert!(str_match("b", "a*b"));
        assert!(!str_match("c", "a*b"));
    }
    #[test]
    fn test_plus() {
        assert!(str_match("aaab", "a+b"));
        assert!(!str_match("b", "a+b"));
    }
    #[test]
    fn test_question() {
        assert!(str_match("ab", "a?b"));
        assert!(str_match("b", "a?b"));
        assert!(!str_match("c", "a?b"));
    }
    #[test]
    fn test_gsub() {
        assert_eq!(str_gsub("foo bar foo", "foo", "baz"), "baz bar baz");
    }
    #[test]
    fn test_gmatch() {
        let s = "foo bar foo baz foo";
        let matches: Vec<_> = str_gmatch(s, "foo").collect();
        assert_eq!(matches, vec![(1, 3), (9, 11), (17, 19)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_str_len() {
        assert_eq!(str_len("hello"), 5);
    }
    #[test]
    fn test_str_sub() {
        assert_eq!(str_sub("abcdef", 2, Some(4)), "bcd");
    }
    #[test]
    fn test_str_reverse() {
        assert_eq!(str_reverse("abc"), "cba");
    }
    #[test]
    fn test_str_lower() {
        assert_eq!(str_lower("ABC"), "abc");
    }
    #[test]
    fn test_str_upper() {
        assert_eq!(str_upper("abc"), "ABC");
    }
    #[test]
    fn test_str_rep() {
        assert_eq!(str_rep("a", 3, Some("-")), "a-a-a");
    }
    #[test]
    fn test_str_byte() {
        assert_eq!(str_byte("abc", 1, Some(2)), vec![97, 98]);
    }
    #[test]
    fn test_str_char() {
        assert_eq!(str_char(&[97, 98, 99]), "abc");
    }
}

#[cfg(test)]
mod ext_tests {
    use super::*;
    #[test]
    fn test_str_find() {
        assert_eq!(str_find("hello world", "world"), Some((7, 11)));
        assert_eq!(str_find("hello", "x"), None);
    }
    #[test]
    fn test_str_match() {
        assert!(str_match("abc", "b"));
        assert!(!str_match("abc", "z"));
    }
    #[test]
    fn test_str_gsub() {
        assert_eq!(str_gsub("aabb", "a", "z"), "zzbb");
    }
    #[test]
    fn test_str_format() {
        assert_eq!(str_format("hi %s!", &["bob"]), "hi bob!");
    }
    #[test]
    fn test_str_dump() {
        assert_eq!(str_dump("abc"), vec![97, 98, 99]);
    }
}

#[cfg(test)]
mod more_ext_tests {
    use super::*;
    #[test]
    fn test_str_gmatch() {
        let s = "foo bar foo baz foo";
        let matches: Vec<_> = str_gmatch(s, "foo").collect();
        assert_eq!(matches, vec![(1, 3), (9, 11), (17, 19)]);
    }
    #[test]
    fn test_str_trim() {
        assert_eq!(str_trim("  hello  "), "hello");
    }
    #[test]
    fn test_str_split() {
        assert_eq!(str_split("a,b,c", Some(",")), vec!["a", "b", "c"]);
        assert_eq!(str_split("a b c", None), vec!["a", "b", "c"]);
    }
}

#[cfg(test)]
mod compact_tests {
    use super::*;
    #[test]
    fn test_stringlib_api() {
        assert_eq!(StringLib::len("abc"), 3);
        assert_eq!(StringLib::sub("abcdef", 2, Some(4)), "bcd");
        assert_eq!(StringLib::reverse("abc"), "cba");
        assert_eq!(StringLib::lower("ABC"), "abc");
        assert_eq!(StringLib::upper("abc"), "ABC");
        assert_eq!(StringLib::rep("a", 3, Some("-")), "a-a-a");
        assert_eq!(StringLib::byte("abc", 1, Some(2)), vec![97, 98]);
        assert_eq!(StringLib::char(&[97, 98, 99]), "abc");
        assert_eq!(StringLib::find("hello world", "world"), Some((7, 11)));
        assert!(StringLib::match_("abc", "b"));
        assert_eq!(StringLib::gsub("aabb", "a", "z"), "zzbb");
        assert_eq!(StringLib::format("hi %s!", &["bob"]), "hi bob!");
        assert_eq!(StringLib::dump("abc"), vec![97, 98, 99]);
        assert_eq!(StringLib::trim("  hi  "), "hi");
        assert_eq!(StringLib::split("a,b", Some(",")), vec!["a", "b"]);
    }
    #[test]
    fn test_stringext_trait() {
        let s = "  abc  ";
        assert_eq!(s.lua_trim(), "abc");
        assert_eq!(s.lua_len(), 7);
        assert_eq!(s.lua_sub(2, Some(4)), " ab");
    }
}
