/// lctype.rs - Character classification and locale handling for Lua-like VM

/// Enum representing character classes (similar to Lua's lctype.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    Control,
    Space,
    Digit,
    Upper,
    Lower,
    Alpha,
    Alnum,
    Print,
    Graph,
    Punct,
    HexDigit,
    Other,
}

/// Returns the character class for a given byte (ASCII)
pub fn char_class(c: u8) -> CharClass {
    match c {
        0..=31 | 127 => CharClass::Control,
        b' ' | b'\t' | b'\n' | b'\r' | 11 | 12 => CharClass::Space,
        b'0'..=b'9' => CharClass::Digit,
        b'A'..=b'Z' => CharClass::Upper,
        b'a'..=b'z' => CharClass::Lower,
        33..=126 => CharClass::Print,
        _ => CharClass::Other,
    }
}

/// Checks if a character is alphabetic
pub fn is_alpha(c: u8) -> bool {
    matches!(char_class(c), CharClass::Upper | CharClass::Lower)
}

/// Checks if a character is a digit
pub fn is_digit(c: u8) -> bool {
    char_class(c) == CharClass::Digit
}

/// Checks if a character is alphanumeric
pub fn is_alnum(c: u8) -> bool {
    is_alpha(c) || is_digit(c)
}

/// Checks if a character is a space
pub fn is_space(c: u8) -> bool {
    char_class(c) == CharClass::Space
}

// Add more functions as needed for your VM's needs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_class() {
        assert_eq!(char_class(b'A'), CharClass::Upper);
        assert_eq!(char_class(b'a'), CharClass::Lower);
        assert_eq!(char_class(b'0'), CharClass::Digit);
        assert_eq!(char_class(b' '), CharClass::Space);
    }

    #[test]
    fn test_is_alpha() {
        assert!(is_alpha(b'A'));
        assert!(is_alpha(b'z'));
        assert!(!is_alpha(b'0'));
    }

    #[test]
    fn test_is_digit() {
        assert!(is_digit(b'5'));
        assert!(!is_digit(b'A'));
    }

    #[test]
    fn test_is_alnum() {
        assert!(is_alnum(b'G'));
        assert!(is_alnum(b'7'));
        assert!(!is_alnum(b'%'));
    }

    #[test]
    fn test_is_space() {
        assert!(is_space(b' '));
        assert!(!is_space(b'A'));
    }
}