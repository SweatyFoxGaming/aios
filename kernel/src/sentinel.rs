//! Sentinel: AI-powered Anomalous Intent Detection for Phoenix OS.
//! Monitors the Synapse IPC bus for deviations from normal behavior.

use crate::println;
use common::security::Token;

/// Triggers a system-wide security lockdown.
pub fn lockdown(trigger_token: &Token, reason: &str) {
    println!("[Sentinel] !!! SECURITY LOCKDOWN TRIGGERED !!!");
    println!("[Sentinel] Triggered by Token: {}", trigger_token.id);
    println!("[Sentinel] Reason: {}", reason);

    // Log to immutable ledger
    crate::audit::log(
        trigger_token,
        crate::safe_alloc::concat2("SECURITY LOCKDOWN: ", reason),
        "Neutralized",
    );

    // Switch OS to restricted state
    crate::ego::set_state(crate::ego::PresenceState::Maintenance);

    // In a real system, we might freeze all non-essential processes here
}

/// Analyzes an intent for anomalous patterns.
pub fn analyze_intent(token: &Token, target: &str, intent: &str) {
    // Basic heuristic: check for decoy "Honey-Intents". Uses
    // safe_alloc::str_eq/contains instead of `==`/`.contains()`: both are
    // PartialEq/Pattern-dispatched and corrupt the return address on this
    // target once reached deep enough in a real boot (see
    // safe_alloc::contains's doc comment).
    if crate::safe_alloc::str_eq(target, "RestrictedDebugService")
        || crate::safe_alloc::contains(intent, "bypass")
    {
        lockdown(token, "Access to Honey-Intent/Restricted Service detected");
    }
}
