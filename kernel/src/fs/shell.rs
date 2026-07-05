//! Phoenix Shell: The native command-line interface for Phoenix OS.

use crate::println;
use crate::print;
use crate::safe_alloc::{first_word, str_eq};

/// Command handler.
///
/// Kept to exactly one `println!` call: a function with more than one
/// `println!` call while a `&str`-derived value (here, `cmd`/`word`) is
/// live in scope corrupts the return address on this target -- verified by
/// extensive bisection elsewhere in this kernel (see `events::publish`'s
/// doc comment). Multi-line output is combined into single calls with
/// embedded newlines instead of separate `println!` calls per line.
///
/// Uses `first_word`/`str_eq` instead of `split_whitespace`/`match` on
/// `&str`: both are `Pattern`/`PartialEq`-dispatched and hit the same
/// corrupted-return-address bug as `str::contains` (see
/// `safe_alloc::contains`'s doc comment) once reached deep enough in a
/// real boot -- this was the actual root cause of this function's crash,
/// not its `println!` count.
pub fn handle_command(cmd: &str) {
    let word = first_word(cmd);
    if word.is_empty() { return; }

    if str_eq(word, "help") {
        println!("Phoenix OS Shell\nAvailable commands: help, clear, info, ls, whoami, exit");
    } else if str_eq(word, "clear") {
        // In a real terminal, we would send ANSI escape codes
        println!("\x1B[2J\x1B[H");
    } else if str_eq(word, "info") {
        println!("Phoenix OS v0.1.0\nTarget: x86_64 Low-End Hardware\nStatus: Cognitive Core Active");
    } else if str_eq(word, "whoami") {
        println!("root@phoenix");
    } else if str_eq(word, "ls") {
        println!("Documents/\nSystem/\nPhoenixFS/");
    } else if str_eq(word, "exit") {
        println!("Shutting down shell...");
    } else {
        println!("Unknown command: {}", word);
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
