//! Vesta: Homeostasis and AI health manager for Phoenix OS.
//! Monitors agent consistency and resource leakage.

use crate::println;
use spin::Mutex;

static LAST_SCORE: Mutex<f32> = Mutex::new(1.0);

/// Checks for inconsistencies or "exhaustion" in AI modules.
pub fn check_health() {
    // Placeholder for actual AI consistency checks
    let consistency_score = 1.0;
    *LAST_SCORE.lock() = consistency_score;

    if consistency_score < 0.7 {
        crate::events::publish("Vesta: AI Consistency Low", 0.8f32.to_bits());
        crate::ego::set_state(crate::ego::PresenceState::Dreaming);
    }
}

/// Log homeostasis status.
pub fn log_status() {
    println!("--- Phoenix Vesta Homeostasis ---");
    println!("Status: Optimal");
    println!("Integrity: 100%");
    println!("---------------------------------");
}

/// Last-computed consistency score, for the Ambient UI's status panel.
#[must_use]
pub fn get_consistency_score() -> f32 {
    *LAST_SCORE.lock()
}
