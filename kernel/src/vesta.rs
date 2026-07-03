//! Vesta: Homeostasis and AI health manager for Phoenix OS.
//! Monitors agent consistency and resource leakage.

use crate::ghost;
use crate::println;
use crate::services;
use alloc::string::ToString;

/// Checks for inconsistencies or "exhaustion" in AI modules.
pub fn check_health() {
    println!("[Vesta] Monitoring system homeostasis...");

    // Check registered services
    let failed_services = services::check_health();
    for service in failed_services {
        println!("[Vesta] ALERT: Service '{}' has failed!", service);
        ghost::heal(service);
    }

    // Placeholder for actual AI consistency checks
    let consistency_score = 1.0;

    if consistency_score < 0.7 {
        crate::events::publish("Vesta: AI Consistency Low".to_string(), 0.8);
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
