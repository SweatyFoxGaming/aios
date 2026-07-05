//! Phoenix Shell: The native command-line interface for Phoenix OS.

use crate::println;
use crate::print;
use alloc::string::String;
use alloc::vec::Vec;

/// Command handler.
///
/// Kept to exactly one `println!` call: a function with more than one
/// `println!` call while a `&str`-derived value (here, `cmd`/`parts`) is
/// live in scope corrupts the return address on this target -- verified by
/// extensive bisection elsewhere in this kernel (see `events::publish`'s
/// doc comment). Multi-line output is combined into single calls with
/// embedded newlines instead of separate `println!` calls per line.
pub fn handle_command(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return; }

    match parts[0] {
        "help" => {
            println!("Phoenix OS Shell\nAvailable commands: help, clear, info, ls, whoami, exit");
        },
        "clear" => {
            // In a real terminal, we would send ANSI escape codes
            println!("\x1B[2J\x1B[H");
        },
        "info" => {
            println!("Phoenix OS v0.1.0\nTarget: x86_64 Low-End Hardware\nStatus: Cognitive Core Active");
        },
        "whoami" => {
            println!("root@phoenix");
        },
        "ls" => {
            println!("Documents/\nSystem/\nPhoenixFS/");
        },
        "exit" => {
            println!("Shutting down shell...");
        },
        _ => {
            println!("Unknown command: {}", parts[0]);
        }
    }
}

/// Simulated shell loop.
pub fn start() {
    println!("--- Phoenix OS Native Shell ---\nType 'help' for a list of commands.");
    println!("phoenix> info");
    handle_command("info");
    println!("phoenix> ls");
    handle_command("ls");
}
