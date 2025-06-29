//! skyla.rs - Skyla stand-alone interpreter (Rust port, forked from Lua)
// Modern, extensible, Rust/D hybrid Lua VM entry point

use crate::lstate::LuaState;
use crate::lobject::LuaValue;
use crate::lauxlib;
use crate::lualib;
use std::env;
use std::process;

const SKYLA_PROGNAME: &str = "skyla";
const SKYLA_INIT_VAR: &str = "SKYLA_INIT";

fn print_usage(badoption: &str) {
    eprint!("{}: ", SKYLA_PROGNAME);
    if badoption.starts_with("-e") || badoption.starts_with("-l") {
        eprintln!("'{}' needs argument", badoption);
    } else {
        eprintln!("unrecognized option '{}'", badoption);
    }
    eprintln!("usage: {} [options] [script [args]]\n\
Available options are:\n\
  -e stat   execute string 'stat'\n\
  -i        enter interactive mode after executing 'script'\n\
  -l mod    require library 'mod' into global 'mod'\n\
  -l g=mod  require library 'mod' into global 'g'\n\
  -v        show version information\n\
  -E        ignore environment variables\n\
  -W        turn warnings on\n\
  --        stop handling options\n\
  -         stop handling options and execute stdin", SKYLA_PROGNAME);
}

fn print_version() {
    println!("Skyla (Rust Lua fork) - version 1.0.0");
}

fn report_error(msg: &str) {
    eprintln!("{}: {}", SKYLA_PROGNAME, msg);
}

fn run_script(state: &mut LuaState, filename: Option<&str>, args: &[String]) -> bool {
    // Load and run a script file, passing args as global 'arg'
    state.set_global("arg", LuaValue::from(args.to_vec()));
    match filename {
        Some(f) => state.do_file(f),
        None => state.do_stdin(),
    }.is_ok()
}

fn run_string(state: &mut LuaState, code: &str) -> bool {
    state.do_string(code).is_ok()
}

/// Extension 1: Add a :q and exit() command to the REPL for quitting
fn register_exit(state: &mut LuaState) {
    state.set_global("exit", LuaValue::Function(Box::new(|_state, _args| {
        println!("[skyla] Exiting REPL.");
        std::process::exit(0);
    })));
}

/// Extension 2: Add :env and env() commands to the REPL for printing environment variables
fn register_env(state: &mut LuaState) {
    state.set_global("env", LuaValue::Function(Box::new(|_state, _args| {
        for (key, value) in std::env::vars() {
            println!("{}={}", key, value);
        }
        Ok(LuaValue::Nil)
    })));
}

/// Extension 3: Add :globals and globals() commands to list all global variables/functions
fn register_globals(state: &mut LuaState) {
    state.set_global("globals", LuaValue::Function(Box::new(|state, _args| {
        let globals = state.get_globals(); // Assumes LuaState::get_globals() returns a Vec<String> or similar
        for name in globals {
            println!("{}", name);
        }
        Ok(LuaValue::Nil)
    })));
}

fn run_repl(state: &mut LuaState) {
    use std::io::{self, Write};
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        line.clear();
        if stdin.read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }
        let trimmed = line.trim();
        if trimmed == ":q" {
            println!("[skyla] Exiting REPL.");
            break;
        }
        if trimmed == ":env" {
            for (key, value) in std::env::vars() {
                println!("{}={}", key, value);
            }
            continue;
        }
        if trimmed == ":globals" {
            let globals = state.get_globals();
            for name in globals {
                println!("{}", name);
            }
            continue;
        }
        if !run_string(state, &line) {
            report_error("Error in input");
        }
    }
}

/// Utility: print a welcome banner with build info and credits
fn print_banner() {
    println!("Skyla VM - Modern Lua Fork (Rust + D)");
    println!("Copyright (c) 2025 Skyla Contributors");
    println!("Build: {} {}", env!("CARGO_PKG_VERSION"), env!("PROFILE"));
    println!("Rust: {}", rustc_version_runtime::version());
    println!("Type 'help()' for usage, or 'exit()' to quit.");
}

/// Utility: provide a help() function in the REPL
fn register_help(state: &mut LuaState) {
    let help_text = "Skyla REPL Help:\n\
  - Type Lua code and press Enter to execute.\n\
  - Use :q or exit() to quit.\n\
  - Use print(...) to display output.\n\
  - Use require('mod') to load modules.\n\
  - Use help() to see this message again.";
    state.set_global("help", LuaValue::Function(Box::new(move |_state, _args| {
        println!("{}", help_text);
        Ok(LuaValue::Nil)
    })));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut state = LuaState::new();
    lualib::open_libs(&mut state);
    register_exit(&mut state);
    register_help(&mut state);
    register_env(&mut state);
    register_globals(&mut state);
    let mut script: Option<&str> = None;
    let mut script_args = Vec::new();
    let mut interactive = false;
    let mut show_version = false;
    let mut ignore_env = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-e" => {
                i += 1;
                if i >= args.len() { print_usage("-e"); process::exit(1); }
                if !run_string(&mut state, &args[i]) { process::exit(1); }
            },
            "-l" => {
                i += 1;
                if i >= args.len() { print_usage("-l"); process::exit(1); }
                // For simplicity, just require the module
                state.require(&args[i]);
            },
            "-i" => interactive = true,
            "-v" => show_version = true,
            "-E" => ignore_env = true,
            "--" => { i += 1; break; },
            "-" => { break; },
            s if s.starts_with('-') => { print_usage(s); process::exit(1); },
            s => { script = Some(s); i += 1; break; }
        }
        i += 1;
    }
    // Remaining args are script args
    script_args.extend_from_slice(&args[i..]);
    if show_version { print_version(); }
    if !ignore_env {
        if let Ok(init) = env::var(SKYLA_INIT_VAR) {
            if init.starts_with('@') {
                let fname = &init[1..];
                if !run_script(&mut state, Some(fname), &script_args) { process::exit(1); }
            } else {
                if !run_string(&mut state, &init) { process::exit(1); }
            }
        }
    }
    if let Some(fname) = script {
        if !run_script(&mut state, Some(fname), &script_args) { process::exit(1); }
        if interactive { run_repl(&mut state); }
    } else if interactive || script.is_none() {
        print_version();
        run_repl(&mut state);
    }
    // Print a warning if any script args are present but no script is given
    if script.is_none() && !script_args.is_empty() {
        eprintln!("[skyla] Warning: script arguments provided but no script specified.");
    }
    // Optionally: allow loading D-based modules via a special flag
    for arg in &args {
        if arg.starts_with("--dmod=") {
            let dmod = &arg[7..];
            // This is a placeholder for D module loading logic
            println!("[skyla] (stub) Would load D module: {}", dmod);
            // You could call into D FFI here
        }
    }
    // Optionally: print a banner or extra info for Skyla
    if show_version && script.is_none() && !interactive {
        println!("Skyla is a modern, extensible Lua fork (Rust + D)");
    }
    // Optionally: run startup hooks or plugins
    // skyla::run_startup_plugins(&mut state); // (stub for future extension)
    // Optionally: print loaded modules and environment info for debugging
    if env::var("SKYLA_DEBUG").is_ok() {
        println!("[skyla] Debug: Loaded modules: {:?}", state.list_loaded_modules());
        println!("[skyla] Debug: Environment: {:?}", env::vars().collect::<Vec<_>>());
    }
    // Optionally: print a goodbye message on exit
    if env::var("SKYLA_GOODBYE").is_ok() {
        println!("[skyla] Goodbye from Skyla!");
    }
    // Optionally: run post-exit hooks or cleanup
    // skyla::run_exit_hooks(&mut state); // (stub for future extension)
}
