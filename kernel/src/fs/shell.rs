//! Phoenix Shell: The native command-line interface for Phoenix OS.

use crate::print;
use crate::println;
use alloc::vec::Vec;

/// Command handler.
pub fn handle_command(cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => {
            println!("Phoenix OS Shell");
            println!("Available commands: help, clear, info, ls, whoami, exit");
        }
        "clear" => {
            // In a real terminal, we would send ANSI escape codes
            println!("\x1B[2J\x1B[H");
        }
        "info" => {
            println!("Phoenix OS v0.1.0");
            println!("Target: x86_64 Low-End Hardware");
            println!("Status: Cognitive Core Active");
        }
        "whoami" => {
            println!("root@phoenix");
        }
        "ls" => {
            println!("Documents/");
            println!("System/");
            println!("PhoenixFS/");
        }
        "exit" => {
            println!("Shutting down shell...");
        }
        _ => {
            println!("Unknown command: {}", parts[0]);
        }
    }
}

/// Simulated shell loop.
pub fn start() {
    println!("--- Phoenix OS Native Shell ---");
    println!("Type 'help' for a list of commands.");

    // Simulate some commands
    print!("phoenix> ");
    println!("info");
    handle_command("info");

    print!("phoenix> ");
    println!("ls");
    handle_command("ls");
}
