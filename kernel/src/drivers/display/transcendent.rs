//! Ambient UI Materialization for Phoenix OS.

use crate::drivers::display::aura;
use crate::println;
use crate::kairos;

/// Materialize a pattern at the user's focus point.
pub fn materialize_at_focus() {
    let (x, y) = kairos::get_attention_point();
    println!(
        "[Aura] Materializing Transcendent UI at attention point ({}, {})",
        x, y
    );
    aura::render_ring(x, y, 30, aura::colors::INTENT_CYAN);
    aura::render_ring(x, y, 45, aura::colors::MATTE_WHITE);
}

/// Initialize the Transcendent UI.
pub fn init() {
    println!("[Aura] Transcendent UI materialization online.");
    materialize_at_focus();
}
