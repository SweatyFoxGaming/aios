//! PS/2 Mouse driver for Phoenix OS.

use crate::println;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

static mut MOUSE_DATA: [u8; 3] = [0; 3];
static mut MOUSE_CYCLE: u8 = 0;

/// Mouse status flags.
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

lazy_static! {
    static ref MOUSE_STATE: Mutex<MouseState> = Mutex::new(MouseState {
        x: 400,
        y: 300,
        left: false,
        right: false,
        middle: false,
    });
}

/// Set once per interrupt when the left button transitions from released
/// to pressed (a "click", not "held") -- `take_click` consumes it exactly
/// once so a single physical click doesn't fire twice.
static PENDING_CLICK: Mutex<Option<(i32, i32)>> = Mutex::new(None);

/// Max poll attempts before giving up on a PS/2 controller status bit. This
/// environment's QEMU PS/2 controller emulation was observed to never clear
/// the expected bit, hanging boot forever with no possible recovery; a
/// bounded retry lets boot continue (without a working mouse) instead.
const MOUSE_WAIT_ATTEMPTS: u32 = 100_000;

fn mouse_wait(a_type: u8) {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mask = if a_type == 0 { 1 } else { 2 };
    for _ in 0..MOUSE_WAIT_ATTEMPTS {
        if (unsafe { status_port.read() } & mask) != mask {
            break;
        }
    }
}

fn mouse_write(a_write: u8) {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);
    mouse_wait(1);
    unsafe { status_port.write(0xD4) };
    mouse_wait(1);
    unsafe { data_port.write(a_write) };
}

fn mouse_read() -> u8 {
    let mut data_port: Port<u8> = Port::new(0x60);
    mouse_wait(0);
    unsafe { data_port.read() }
}

/// Initialize the PS/2 mouse.
pub fn init() {
    println!("[Nerve] Initializing PS/2 Mouse...");
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    mouse_wait(1);
    unsafe { status_port.write(0xA8) }; // Enable auxiliary device

    mouse_wait(1);
    unsafe { status_port.write(0x20) }; // Get command byte
    let mut status = mouse_read() | 2;
    mouse_wait(1);
    unsafe { status_port.write(0x60) }; // Set command byte
    mouse_wait(1);
    unsafe { data_port.write(status) };

    mouse_write(0xF6); // Set default settings
    let _ = mouse_read(); // Acknowledge

    mouse_write(0xF4); // Enable data reporting
    let _ = mouse_read(); // Acknowledge

    println!("[Nerve] Mouse initialized.");
}

/// Handle a mouse interrupt.
pub fn handle_interrupt() {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);
    let status = unsafe { status_port.read() };
    if (status & 0x01) != 0 && (status & 0x20) != 0 {
        let data = unsafe { data_port.read() };
        unsafe {
            match MOUSE_CYCLE {
                0 => {
                    MOUSE_DATA[0] = data;
                    if (data & 0x08) != 0 {
                        MOUSE_CYCLE = 1;
                    }
                }
                1 => {
                    MOUSE_DATA[1] = data;
                    MOUSE_CYCLE = 2;
                }
                2 => {
                    MOUSE_DATA[2] = data;
                    MOUSE_CYCLE = 0;

                    let left = (MOUSE_DATA[0] & 0x01) != 0;
                    let right = (MOUSE_DATA[0] & 0x02) != 0;
                    let middle = (MOUSE_DATA[0] & 0x04) != 0;

                    let mut dx = MOUSE_DATA[1] as i32;
                    let mut dy = MOUSE_DATA[2] as i32;

                    if (MOUSE_DATA[0] & 0x10) != 0 {
                        dx -= 256;
                    }
                    if (MOUSE_DATA[0] & 0x20) != 0 {
                        dy -= 256;
                    }

                    let mut state = MOUSE_STATE.lock();
                    let was_left = state.left;
                    // PS/2 reports +y as "up" (screen coordinates increase
                    // downward), so dy is subtracted rather than added.
                    state.x = (state.x + dx).clamp(0, 1023);
                    state.y = (state.y - dy).clamp(0, 767);
                    state.left = left;
                    state.right = right;
                    state.middle = middle;
                    let moved = dx != 0 || dy != 0;
                    let (cx, cy) = (state.x, state.y);
                    drop(state);

                    if left && !was_left {
                        *PENDING_CLICK.lock() = Some((cx, cy));
                    }
                    if moved || left != was_left {
                        crate::drivers::display::ambient_ui::on_mouse_activity();
                    }
                }
                _ => MOUSE_CYCLE = 0,
            }
        }
    }
}

/// Current tracked cursor position and button state.
#[must_use]
pub fn get_state() -> MouseState {
    let state = MOUSE_STATE.lock();
    MouseState {
        x: state.x,
        y: state.y,
        left: state.left,
        right: state.right,
        middle: state.middle,
    }
}

/// Consume the pending click (if any) -- returns `Some((x, y))` at most
/// once per rising edge of the left button.
pub fn take_click() -> Option<(i32, i32)> {
    PENDING_CLICK.lock().take()
}
