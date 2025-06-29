/// idebug.rs - Internal debug utilities for Lua-like VM in Rust

use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(true);

/// Example: Internal function to print the current call stack.
/// In a real implementation, this would walk the VM's call stack and print details.
pub fn print_call_stack() {
    // Placeholder: print a message
    println!("[idebug] Call stack (not implemented)");
}

/// Example: Internal function to print the current instruction pointer.
/// In a real implementation, this would print the current instruction index or address.
pub fn print_instruction_pointer(ip: usize) {
    println!("[idebug] Instruction pointer: {}", ip);
}

/// Example: Internal function to print the value of a register.
/// In a real implementation, this would print the value stored in a VM register.
pub fn print_register_value(reg: usize, value: &str) {
    println!("[idebug] Register[{}] = {}", reg, value);
}

/// Enable debug logging.
pub fn enable_debug() {
    DEBUG_ENABLED.store(true, Ordering::Relaxed);
}

/// Disable debug logging.
pub fn disable_debug() {
    DEBUG_ENABLED.store(false, Ordering::Relaxed);
}

/// Example: Internal function to log debug messages.
pub fn log_debug_message(msg: &str) {
    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        println!("[idebug] {}", msg);
    }
}

// Add more internal debug helpers as needed...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_call_stack() {
        print_call_stack();
    }

    #[test]
    fn test_log_debug_message() {
        enable_debug();
        log_debug_message("Test message");
        disable_debug();
        log_debug_message("This should not appear");
        enable_debug();
    }

    #[test]
    fn test_print_instruction_pointer() {
        print_instruction_pointer(42);
    }

    #[test]
    fn test_print_register_value() {
        print_register_value(3, "0xDEADBEEF");
    }

    #[test]
    fn test_enable_disable_debug() {
        enable_debug();
        assert!(super::DEBUG_ENABLED.load(std::sync::atomic::Ordering::Relaxed));
        disable_debug();
        assert!(!super::DEBUG_ENABLED.load(std::sync::atomic::Ordering::Relaxed));
    }
}