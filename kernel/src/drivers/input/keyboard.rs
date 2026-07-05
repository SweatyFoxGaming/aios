//! Nerve: Keyboard and input system for Phoenix OS.
//! Translates PS/2 scancodes into intents and Synapse messages.

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

    /// Buffer for the Chat panel's input before Enter is pressed.
    /// Preallocated so `.push` below never needs to grow the buffer --
    /// growing an existing heap allocation crashes on this target (see
    /// safe_alloc.rs).
    static ref INPUT_BUFFER: Mutex<String> = Mutex::new(String::with_capacity(256));
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
                    if character == '`' {
                        // Toggle Ghost Shell with backtick (unrelated to
                        // the Ambient UI; kept as-is).
                        crate::drivers::display::toggle_ghost_shell();
                    } else {
                        crate::drivers::display::ambient_ui::on_key(Some(character), false);
                    }
                }
                DecodedKey::RawKey(pc_keyboard::KeyCode::Escape) => {
                    crate::drivers::display::ambient_ui::on_key(None, true);
                }
                DecodedKey::RawKey(_) => {}
            }
        }
    }
}

/// Handle one character typed while the Chat panel has focus. Called from
/// `ambient_ui::on_key`. On Enter, dispatches the accumulated buffer to
/// the Hermes intent parser (the Chat panel renders the result in its
/// scrollback).
pub fn chat_panel_key(character: char) {
    if character == '\n' {
        crate::drivers::display::ambient_ui::submit_chat_message();
    } else if INPUT_BUFFER.lock().len() < 256 {
        INPUT_BUFFER.lock().push(character);
    }
}

/// Take and clear the current Chat panel input buffer contents.
pub fn take_chat_input() -> String {
    let mut buf = INPUT_BUFFER.lock();
    let taken = crate::safe_alloc::to_string(&buf);
    buf.clear();
    taken
}
