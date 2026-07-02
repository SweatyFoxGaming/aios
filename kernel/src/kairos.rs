//! Kairòs: The context engine for Phoenix OS.
//! Tracks the "Now" - active intents, system mood, and attention focus.

use crate::println;
use alloc::string::String;
use alloc::string::ToString;
use lazy_static::lazy_static;
use spin::Mutex;

/// Represents the current contextual focus of the system.
#[derive(Debug, Clone)]
pub struct Context {
    /// The current goal or active intent.
    pub active_intent: String,
    /// System mood based on resource pressure and activity.
    pub system_mood: String,
    /// The specific module or task currently holding attention.
    pub attention_focus: &'static str,
}

lazy_static! {
    static ref CONTEXT: Mutex<Context> = Mutex::new(Context {
        active_intent: String::from("Initialization"),
        system_mood: String::from("Calm"),
        attention_focus: "KernelCore",
    });
}

/// Update the active intent in the context engine.
pub fn set_intent(intent: &str) {
    let mut ctx = CONTEXT.lock();
    ctx.active_intent = intent.to_string();

    crate::events::publish(alloc::format!("Kairòs: Intent shift -> {intent}"), 0.7);
}

/// Update the system mood.
pub fn set_mood(mood: String) {
    let mut ctx = CONTEXT.lock();
    ctx.system_mood = mood;
}

/// Get a copy of the current context.
#[must_use]
pub fn get_context() -> Context {
    CONTEXT.lock().clone()
}

/// Log current context status.
pub fn log_status() {
    let ctx = CONTEXT.lock();
    println!("--- Kairòs Context Engine ---");
    println!("Active Intent: {}", ctx.active_intent);
    println!("System Mood:   {}", ctx.system_mood);
    println!("Attention:     {}", ctx.attention_focus);
    println!("-----------------------------");
}
