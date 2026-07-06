//! Hermes: Intent parsing and orchestration for JARVIS.

use crate::println;
use crate::safe_alloc::{contains, str_eq, to_string};
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

    // In a real system, this would use a local LLM or NLP model. Uses
    // safe_alloc::contains instead of `str::contains`: the latter's
    // `Pattern`-dispatched implementation corrupts the return address on
    // this target once reached deep enough in a real boot -- this was the
    // actual root cause of this function's crash (see
    // safe_alloc::contains's doc comment), not anything about `parse`
    // itself (which is otherwise correct, as isolated testing showed).
    let action = if contains(input, "research") {
        "KnowledgeQuery"
    } else if contains(input, "fix") || contains(input, "heal") {
        "SelfRepair"
    } else {
        "GeneralInteraction"
    };

    Intent {
        raw: to_string(input),
        action: to_string(action),
        significance: 0.8,
    }
}

/// Dispatch an intent to the appropriate system service.
pub fn dispatch(intent: &Intent) {
    println!("[Hermes] Dispatching action: {}", intent.action);

    if str_eq(&intent.action, "SelfRepair") {
        crate::ghost::heal("TargetedService");
    } else {
        synapse::send(Message::new("Hermes", "CognitiveCore", to_string(&intent.action)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression guard for the bug root-caused in commit e052c19: `parse`
    // only crashed once reached deep enough in a real boot, not in isolated
    // testing -- comparisons here go through `str_eq`, never raw `==`, per
    // safe_alloc's documented constraint.

    #[test_case]
    fn parse_classifies_research_intent() {
        let intent = parse("research solid state batteries");
        assert!(str_eq(&intent.action, "KnowledgeQuery"));
    }

    #[test_case]
    fn parse_classifies_repair_intent() {
        let intent = parse("please fix the display");
        assert!(str_eq(&intent.action, "SelfRepair"));
    }

    #[test_case]
    fn parse_falls_back_to_general_interaction() {
        let intent = parse("hello there");
        assert!(str_eq(&intent.action, "GeneralInteraction"));
    }
}
