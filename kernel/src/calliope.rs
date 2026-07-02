//! Calliope: WASM Skill Runtime for Phoenix OS.

use crate::drivers::audio::speaker::{self, HarmonicPhrase};
use crate::drivers::display::engine::{self, Pattern};
use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use common::synapse::Prosody;

/// Represents a JARVIS Skill (WASM-based).
pub struct Skill {
    /// The name of the skill.
    pub name: String,
    /// The WASM bytecode for the skill.
    pub bytecode: Vec<u8>,
}

/// Initialize the Skill Runtime.
pub fn init() {
    println!("[Calliope] WASM Skill Runtime online.");
}

/// Execute a WASM skill in a sandbox.
pub fn execute(skill: &Skill) -> Result<(), &'static str> {
    println!(
        "[Calliope] Executing skill '{}' ({} bytes) in WASM sandbox...",
        skill.name,
        skill.bytecode.len()
    );

    // In a real implementation, we would use a library like wasmi
    // to execute the bytecode and provide host imports for Synapse.
    println!("[Calliope] Skill '{}' completed successfully.", skill.name);
    Ok(())
}

/// Orchestrate JARVIS "Speech" through harmonic and visual feedback.
pub fn speak(text: &str, prosody: Prosody) {
    println!("[JARVIS] Speaking (Prosody: {:?}): \"{}\"", prosody, text);

    // 1. Set Visual Pattern
    engine::materialize(Pattern::VoiceActivity);

    // 2. Play Harmonic Cadence based on Prosody
    match prosody {
        Prosody::Calm => speaker::play_phrase(HarmonicPhrase::StartingTask),
        Prosody::Urgent => speaker::play_phrase(HarmonicPhrase::Alert),
        Prosody::Empathetic => speaker::play_phrase(HarmonicPhrase::Thinking), // Soft pulse
        Prosody::Thinking => speaker::play_phrase(HarmonicPhrase::Thinking),
    }

    // 3. Return to Idle/Normal after "speaking"
    engine::materialize(Pattern::Idle);
}
