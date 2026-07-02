//! Lethe: Significance-based memory pruning for Phoenix OS.

use crate::mnemosyne;
use crate::println;

/// Pruning configuration.
pub struct PruningPolicy {
    /// Significance threshold (0.0 to 1.0). Data below this may be pruned.
    pub threshold: f32,
}

/// Run a pruning cycle based on memory pressure.
pub fn prune(pressure_level: f32) {
    println!(
        "[Lethe] Memory pressure at {:.2}. Initiating pruning cycle...",
        pressure_level
    );

    let threshold = pressure_level * 0.5;
    let policy = PruningPolicy { threshold };

    println!("[Lethe] Policy: Decay 0.1, Prune < {:.2}", policy.threshold);

    // 1. Decay all knowledge nodes
    mnemosyne::decay_significance(0.1);

    // 2. Prune nodes below threshold
    let removed = mnemosyne::prune_nodes(policy.threshold);

    println!(
        "[Lethe] Pruning complete. Removed {} low-significance nodes.",
        removed
    );
}

/// Log the state of Lethe.
pub fn log_status() {
    println!("[Lethe] Engine active. Policy: Adaptive Significance.");
}
