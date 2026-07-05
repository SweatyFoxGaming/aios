//! Lethe: Significance-based memory pruning for Phoenix OS.

use crate::println;
use crate::mnemosyne;
use alloc::vec::Vec;

/// Pruning configuration.
pub struct PruningPolicy {
    /// Significance threshold (0.0 to 1.0). Data below this may be pruned.
    pub threshold: f32,
}

/// Run a pruning cycle based on memory pressure.
///
/// Kept to exactly one `println!` call, and avoids `{:.N}` precision float
/// formatting: a loop repeatedly using `{:.2}` on an f32 hangs on this
/// target (verified in `events::list_events` -- a single, non-looped
/// `{:.2}` call elsewhere works fine, so this is conservative rather than
/// strictly required here, but this function's un-exercised status made it
/// easy to miss a regression in). Plain `{}` Display formatting is safe.
pub fn prune(pressure_level: f32) {
    let policy = PruningPolicy {
        threshold: pressure_level * 0.5,
    };

    println!(
        "[Lethe] Memory pressure at {pressure_level}. Pruning nodes with significance < {}. Recovered simulated 128 MB.",
        policy.threshold
    );
}

/// Log the state of Lethe.
pub fn log_status() {
    println!("[Lethe] Engine active. Policy: Adaptive Significance.");
}
