//! PS/2 Mouse driver for Phoenix OS.

use crate::println;
use x86_64::instructions::port::Port;

static mut MOUSE_DATA: [u8; 3] = [0; 3];
static mut MOUSE_CYCLE: u8 = 0;

/// Mouse status flags.
pub struct MouseState {
    /// X-axis movement.
    pub x: i32,
    /// Y-axis movement.
    pub y: i32,
    /// Left button pressed.
    pub left: bool,
    /// Right button pressed.
    pub right: bool,
    /// Middle button pressed.
    pub middle: bool,
}

fn mouse_wait(a_type: u8) {
    let mut status_port: Port<u8> = Port::new(0x64);
    if a_type == 0 {
        while (unsafe { status_port.read() } & 1) == 1 {}
    } else {
        while (unsafe { status_port.read() } & 2) == 2 {}
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
    let status = mouse_read() | 2;
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

                    let _left = (MOUSE_DATA[0] & 0x01) != 0;
                    let _right = (MOUSE_DATA[0] & 0x02) != 0;
                    let _middle = (MOUSE_DATA[0] & 0x04) != 0;

                    let mut x = MOUSE_DATA[1] as i32;
                    let mut y = MOUSE_DATA[2] as i32;

                    if (MOUSE_DATA[0] & 0x10) != 0 {
                        x -= 256;
                    }
                    if (MOUSE_DATA[0] & 0x20) != 0 {
                        y -= 256;
                    }

                    let _ = x;
                    let _ = y;

                    // Here we would dispatch a mouse event
                    // println!("Mouse Move: dx={}, dy={}", x, y);
                }
                _ => MOUSE_CYCLE = 0,
            }
        }
    }
}
