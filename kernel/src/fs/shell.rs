//! Phoenix Shell: The native command-line interface for Phoenix OS.

use crate::println;
use crate::safe_alloc::{first_word, str_eq};
use alloc::string::String;

/// Command handler. Returns its output as a `String` instead of
/// `println!`-ing directly, so both `start()`'s boot demo and the
/// Ambient UI's Command panel (`drivers::display::ambient_ui`) can use
/// the same logic and render/print the result themselves.
///
/// Uses `first_word`/`str_eq` instead of `split_whitespace`/`match` on
/// `&str`: both are `Pattern`/`PartialEq`-dispatched and corrupt the
/// return address on this target once reached deep enough in a real
/// boot (see `safe_alloc::contains`'s doc comment).
pub fn handle_command(cmd: &str) -> String {
    let word = first_word(cmd);
    if word.is_empty() {
        return crate::safe_alloc::to_string("");
    }

    if str_eq(word, "help") {
        crate::safe_alloc::to_string("Available: help, clear, info, ls, whoami, exit")
    } else if str_eq(word, "clear") {
        crate::safe_alloc::to_string("")
    } else if str_eq(word, "info") {
        crate::safe_alloc::to_string("Phoenix OS v0.1.0 - Cognitive Core Active")
    } else if str_eq(word, "whoami") {
        crate::safe_alloc::to_string("root@phoenix")
    } else if str_eq(word, "ls") {
        crate::safe_alloc::to_string("Documents/ System/ PhoenixFS/")
    } else if str_eq(word, "exit") {
        crate::safe_alloc::to_string("Shutting down shell...")
    } else {
        crate::safe_alloc::concat2("Unknown command: ", word)
    }
}

/// Simulated shell loop.
pub fn start() {
    println!("--- Phoenix OS Native Shell ---\nType 'help' for a list of commands.");
    println!("phoenix> info");
    println!("{}", handle_command("info"));
    println!("phoenix> ls");
    println!("{}", handle_command("ls"));
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression guard for the boot-only shell crash fixed alongside
    // hermes::parse in commit e052c19 -- comparisons go through `str_eq`,
    // never raw `==`, per safe_alloc's documented constraint.

    #[test_case]
    fn handle_command_help_lists_commands() {
        let out = handle_command("help");
        assert!(str_eq(&out, "Available: help, clear, info, ls, whoami, exit"));
    }

    #[test_case]
    fn handle_command_unknown_echoes_input() {
        let out = handle_command("frobnicate");
        assert!(str_eq(&out, "Unknown command: frobnicate"));
    }

    #[test_case]
    fn handle_command_empty_input_returns_empty() {
        assert!(handle_command("").is_empty());
        assert!(handle_command("   ").is_empty());
    }
}
