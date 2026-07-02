//! Ego: The self-awareness and identity service for Phoenix OS.

use crate::println;
use common::security::Token;
use lazy_static::lazy_static;
use spin::Mutex;

/// The global state of the Phoenix OS presence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    /// System is initializing.
    Initializing,
    /// System is idle, awaiting intent (Ambient UI mode).
    Idle,
    /// System is actively processing a user task.
    Active,
    /// System is performing internal optimizations (Maintenance).
    Dreaming,
    /// System is in a restricted recovery state.
    Maintenance,
}

struct Ego {
    state: PresenceState,
    next_token_id: u64,
}

lazy_static! {
    static ref EGO: Mutex<Ego> = Mutex::new(Ego {
        state: PresenceState::Initializing,
        next_token_id: 1, // 0 is reserved for KernelCore
    });
}

/// Get the current state of the Phoenix presence.
#[must_use]
pub fn get_state() -> PresenceState {
    EGO.lock().state
}

/// Set the current state of the Phoenix presence.
pub fn set_state(new_state: PresenceState) {
    let mut ego = EGO.lock();
    let old_state = ego.state;
    ego.state = new_state;

    crate::events::publish(
        alloc::format!("Ego: State transition {old_state:?} -> {new_state:?}"),
        0.8,
    );
}

/// Issue a new capability token.
#[must_use]
pub fn issue_token() -> Token {
    let mut ego = EGO.lock();
    let token = Token::empty(ego.next_token_id);
    ego.next_token_id += 1;
    token
}

/// Log current identity status.
pub fn log_status() {
    let ego = EGO.lock();
    println!("--- Phoenix Ego Identity ---");
    println!("Presence State: {:?}", ego.state);
    println!("Tokens Issued: {}", ego.next_token_id);
    println!("----------------------------");
}
