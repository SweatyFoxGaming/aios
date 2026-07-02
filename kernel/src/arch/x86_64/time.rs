//! Chrono: Precision timing foundation for Phoenix OS.
//! Implements a monotonic clock using the PIT (Programmable Interval Timer).

use spin::Mutex;
use x86_64::instructions::port::Port;

/// PIT command port.
const PIT_COMMAND: u16 = 0x43;
/// PIT Channel 0 data port.
const PIT_CHANNEL_0: u16 = 0x40;
/// Frequency of the PIT (1.193182 MHz).
const PIT_FREQUENCY: u32 = 1_193_182;

static TICKS: Mutex<u64> = Mutex::new(0);

/// Initialize the PIT timer to a specific frequency (Hz).
pub fn init(frequency: u32) {
    let divisor = PIT_FREQUENCY / frequency;

    let mut cmd_port = Port::new(PIT_COMMAND);
    let mut data_port = Port::new(PIT_CHANNEL_0);

    unsafe {
        // Mode 3: Square wave generator
        cmd_port.write(0x36u8);
        data_port.write((divisor & 0xFF) as u8);
        data_port.write(((divisor >> 8) & 0xFF) as u8);
    }
}

/// Increment the monotonic tick counter.
/// This should be called by the timer interrupt handler.
pub fn tick() {
    let mut ticks = TICKS.lock();
    *ticks += 1;
}

/// Returns the number of ticks since boot.
#[must_use]
pub fn get_uptime() -> u64 {
    *TICKS.lock()
}
