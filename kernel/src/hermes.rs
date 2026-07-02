//! Hermes: Raw intent parser for Phoenix OS.
//! Translates input into structured Synapse messages.

use crate::println;
use alloc::string::String;
use common::synapse::Message;

/// Possible user intents.
#[derive(Debug, Clone)]
pub enum Intent {
    /// Request to research a topic.
    Research(String),
    /// Request to perform a system maintenance task.
    Maintain(String),
    /// Request to develop code.
    Develop(String),
    /// Unknown or unparsed intent.
    Unknown(String),
}

/// Parses a raw string into a structured Intent.
#[must_use]
pub fn parse(input: &str) -> Intent {
    if input.contains("research") {
        Intent::Research(String::from(input))
    } else if input.contains("clean") || input.contains("fix") {
        Intent::Maintain(String::from(input))
    } else if input.contains("build") || input.contains("code") {
        Intent::Develop(String::from(input))
    } else {
        Intent::Unknown(String::from(input))
    }
}

/// Dispatches an intent via the Synapse bus.
pub fn dispatch(intent: &Intent) {
    let (target, description) = match *intent {
        Intent::Research(ref s) => ("ResearchAgent", s),
        Intent::Maintain(ref s) => ("MaintenanceAgent", s),
        Intent::Develop(ref s) => ("CodingAgent", s),
        Intent::Unknown(ref s) => ("Commander", s),
    };

    println!("[Hermes] Dispatching intent: {target} -> {description}");

    // Update context engine
    crate::kairos::set_intent(description);

    // Send Synapse message
    crate::synapse::send(Message::new("Hermes", target, description.clone()));
}
