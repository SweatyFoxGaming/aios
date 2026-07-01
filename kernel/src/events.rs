//! Neural-inspired Event Bus for Phoenix OS.
//! Every event has a "Significance Score" that JARVIS uses for filtering.

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

/// Significance score for an event (0.0 to 1.0).
pub type Significance = f32;

/// A system-wide event.
#[derive(Debug, Clone)]
pub struct Event {
    /// The name/type of the event.
    pub name: String,
    /// The importance of the event.
    pub significance: Significance,
    /// Optional structured data (placeholder).
    pub data: Option<String>,
}

lazy_static! {
    static ref EVENT_BUS: Mutex<Vec<Event>> = Mutex::new(Vec::new());
}

/// Minimum significance threshold for logging/alerting.
const LOG_THRESHOLD: Significance = 0.5;

/// Publish an event to the neural bus.
pub fn publish(name: String, significance: Significance) {
    if significance >= LOG_THRESHOLD {
        println!(
            "[Neural Bus] High-Significance Event: {} ({})",
            name, significance
        );
    }

    let event = Event {
        name,
        significance,
        data: None,
    };

    EVENT_BUS.lock().push(event);
}

/// List recently published events.
pub fn list_events() {
    let bus = EVENT_BUS.lock();
    println!("--- Neural Event Bus Activity ---");
    for event in bus.iter() {
        println!("[{:.2}] {}", event.significance, event.name);
    }
    println!("---------------------------------");
}
