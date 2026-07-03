//! Ambient UI Engine for Phoenix OS.

use crate::drivers::display::aura;
use crate::println;

/// Types of materialization patterns.
pub enum Pattern {
    /// JARVIS Concentric Rings.
    ConcentricRings,
    /// Intent Materialization.
    Materialize,
    /// Alert / Warning Pulse.
    Pulse,
    /// JARVIS Voice Activity (Melodic pulse).
    VoiceActivity,
    /// Idle State (Emblem).
    Idle,
}

/// The Ambient UI Engine state.
pub struct Engine {
    /// Currently active materialization pattern.
    pub active_pattern: Pattern,
    /// The user's current focus/attention point (x, y).
    pub focus_point: (usize, usize),
}

static mut ENGINE: Engine = Engine {
    active_pattern: Pattern::ConcentricRings,
    focus_point: (400, 300),
};

/// Initialize the Ambient UI Engine.
pub fn init() {
    println!("[Aura] Ambient UI Engine online.");
    materialize(Pattern::ConcentricRings);
}

/// Materialize a pattern.
pub fn materialize(pattern: Pattern) {
    unsafe {
        ENGINE.active_pattern = pattern;
        match ENGINE.active_pattern {
            Pattern::ConcentricRings => {
                aura::render_ring(
                    ENGINE.focus_point.0,
                    ENGINE.focus_point.1,
                    40,
                    aura::colors::INTENT_CYAN,
                );
                aura::render_ring(
                    ENGINE.focus_point.0,
                    ENGINE.focus_point.1,
                    60,
                    aura::colors::INTENT_CYAN,
                );
            }
            Pattern::Materialize => {
                aura::clear_screen();
                aura::materialize_workspace("General Purpose");
            }
            Pattern::Pulse => {
                aura::render_ring(
                    ENGINE.focus_point.0,
                    ENGINE.focus_point.1,
                    50,
                    aura::colors::INTENT_CYAN,
                );
            }
            Pattern::VoiceActivity => {
                aura::render_ring(
                    ENGINE.focus_point.0,
                    ENGINE.focus_point.1,
                    45,
                    aura::colors::MATTE_WHITE,
                );
                aura::render_ring(
                    ENGINE.focus_point.0,
                    ENGINE.focus_point.1,
                    55,
                    aura::colors::INTENT_CYAN,
                );
            }
            Pattern::Idle => {
                aura::render_emblem();
            }
        }
    }
}
