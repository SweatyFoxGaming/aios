//! Hermes: Intent parsing and orchestration for JARVIS.

use crate::println;
use crate::synapse;
use alloc::string::String;
use common::synapse::Message;

/// Represents a parsed intent.
pub struct Intent {
    pub raw: String,
    pub action: String,
    pub significance: f32,
}

/// Parse raw input into a structured intent.
pub fn parse(input: &str) -> Intent {
    println!("[Hermes] Parsing intent: '{}'", input);

    // In a real system, this would use a local LLM or NLP model
    let action = if input.contains("research") {
        "KnowledgeQuery"
    } else if input.contains("fix") || input.contains("heal") {
        "SelfRepair"
    } else {
        "GeneralInteraction"
    };

    Intent {
        raw: String::from(input),
        action: String::from(action),
        significance: 0.8,
    }
}

/// Dispatch an intent to the appropriate system service.
pub fn dispatch(intent: &Intent) {
    println!("[Hermes] Dispatching action: {}", intent.action);

    if intent.action == "SelfRepair" {
        crate::ghost::heal("TargetedService");
    } else {
        synapse::send(Message::new("Hermes", "CognitiveCore", intent.action.clone()));
    }
}
