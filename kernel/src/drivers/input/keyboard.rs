//! Nerve: Keyboard and input system for Phoenix OS.
//! Translates PS/2 scancodes into intents and Synapse messages.

use crate::println;
use alloc::string::String;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

lazy_static! {
    /// Global keyboard state.
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore
    ));

    /// Buffer for raw input before Enter is pressed.
    static ref INPUT_BUFFER: Mutex<String> = Mutex::new(String::new());
}

/// Handle a keyboard interrupt and parse the scancode.
pub fn handle_interrupt() {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    if character == '\n' {
                        dispatch_buffer();
                    } else if character == '`' {
                        // Toggle Ghost Shell with backtick
                        crate::drivers::display::toggle_ghost_shell();
                    } else {
                        INPUT_BUFFER.lock().push(character);
                        // Echo to serial for now
                        crate::print!("{character}");
                    }
                }
                DecodedKey::RawKey(key) => println!("Raw Key: {key:?}"),
            }
        }
    }
}

/// Dispatches the current input buffer to the Hermes Intent Parser.
fn dispatch_buffer() {
    let mut buffer = INPUT_BUFFER.lock();
    if !buffer.is_empty() {
        println!("\n[Nerve] Intent Captured: {buffer}");
        let intent = crate::hermes::parse(&buffer);
        crate::hermes::dispatch(&intent);
        buffer.clear();
    }
}
