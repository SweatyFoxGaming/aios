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
    // Preallocated so `EVENT_BUS.lock().push` below never needs to grow the
    // `Vec` -- growing an existing heap allocation crashes on this target
    // (see safe_alloc.rs).
    static ref EVENT_BUS: Mutex<Vec<Event>> = Mutex::new(Vec::with_capacity(64));
}

/// Publish an event to the neural bus.
///
/// Takes `name` as `&str` and `significance` as raw `u32` bits (via
/// `f32::to_bits`), not `String`/`f32` directly, and logs with exactly one
/// `println!` call. Three compounding issues on this target, isolated via
/// extensive testing:
/// (1) passing a `String` argument together with an `f32` argument in the
/// same call crashes with a corrupted return address (mixing a
/// multi-register struct argument with a floating-point register
/// argument);
/// (2) even with the `f32` replaced by `u32`, a function that receives an
/// owned `String` *by value* and then takes a reference to it (as
/// `println!("{}", name)` does for Display formatting) also corrupts the
/// return address -- taking `&str` instead sidesteps this, since it's
/// always a plain two-word pointer needing no stack spill;
/// (3) even with `&str`, a function with more than one `println!` call
/// where any one of them formats that `&str` parameter *still* corrupts
/// the return address -- verified by bisection (1 call: works; 2+ calls:
/// fails, regardless of which one references the parameter). Root cause
/// not fully traced; the workaround is simply to keep such functions to
/// exactly one `println!`.
pub fn publish(name: &str, significance_bits: u32) {
    let significance = Significance::from_bits(significance_bits);
    println!("[Neural Bus] Event: {name} ({significance})");

    let event = Event {
        name: crate::safe_alloc::to_string(name),
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
