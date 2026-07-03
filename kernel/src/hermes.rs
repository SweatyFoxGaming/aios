//! Hermes: Intent parsing and orchestration for JARVIS.

use crate::println;
use crate::synapse;
use alloc::string::String;
use common::synapse::Message;

/// Represents a parsed intent.
pub struct Intent {
    /// The raw input string.
    pub raw: String,
    /// The parsed action/command.
    pub action: String,
    /// The calculated significance of the intent (0.0 to 1.0).
    pub significance: f32,
}

/// A keyword-to-action mapping.
struct LexiconEntry {
    keyword: &'static str,
    action: &'static str,
}

const LEXICON: &[LexiconEntry] = &[
    LexiconEntry { keyword: "research", action: "KnowledgeQuery" },
    LexiconEntry { keyword: "analyze", action: "KnowledgeQuery" },
    LexiconEntry { keyword: "fix", action: "SelfRepair" },
    LexiconEntry { keyword: "heal", action: "SelfRepair" },
    LexiconEntry { keyword: "patch", action: "SelfRepair" },
    LexiconEntry { keyword: "optimize", action: "SelfOptimization" },
    LexiconEntry { keyword: "clear", action: "InterfaceReset" },
    LexiconEntry { keyword: "who", action: "IdentityQuery" },
];

/// Parse raw input into a structured intent.
pub fn parse(input: &str) -> Intent {
    println!("[Hermes] Parsing intent: '{}'", input);
    let normalized = input.to_lowercase();

    // 1. Check the Lexicon Registry
    for entry in LEXICON {
        if normalized.contains(entry.keyword) {
            return Intent {
                raw: String::from(input),
                action: String::from(entry.action),
                significance: 0.9,
            };
        }
    }

    // 2. Semantic Fallback: Check if the input contains known concepts from Mnemosyne
    // In a real system, we'd iterate through knowledge nodes.
    // For now, we simulate a "Learning Mode" trigger if unknown.
    let action = if normalized.len() > 3 {
        "ConceptDiscovery"
    } else {
        "GeneralInteraction"
    };

    Intent {
        raw: String::from(input),
        action: String::from(action),
        significance: 0.5,
    }
}

/// Dispatch an intent to the appropriate system service.
pub fn dispatch(intent: &Intent) {
    println!("[Hermes] Dispatching action: {}", intent.action);

    match intent.action.as_str() {
        "SelfRepair" => {
            crate::ghost::heal("TargetedService");
        }
        "ConceptDiscovery" => {
            // Trigger the JARVIS learning loop for unknown concepts
            crate::calliope::learn_concept(&intent.raw);
        }
        _ => {
            synapse::send(Message::new(
                "Hermes",
                "CognitiveCore",
                intent.action.clone(),
            ));
        }
    }
}
