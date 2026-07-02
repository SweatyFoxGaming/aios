//! Calliope: Basic audio feedback (PC Speaker) for Phoenix OS.

use x86_64::instructions::port::Port;

/// PC Speaker data port.
const SPEAKER_PORT: u16 = 0x61;
/// PIT Channel 2 data port.
const PIT_CHANNEL_2: u16 = 0x42;
/// PIT command port.
const PIT_COMMAND: u16 = 0x43;

/// Plays a tone on the PC speaker.
pub fn play_tone(frequency: u32) {
    if frequency == 0 {
        return;
    }

    let divisor = 1_193_180 / frequency;
    let mut cmd_port = Port::new(PIT_COMMAND);
    let mut data_port = Port::new(PIT_CHANNEL_2);
    let mut speaker_port = Port::new(SPEAKER_PORT);

    unsafe {
        cmd_port.write(0xB6u8);
        data_port.write((divisor & 0xFF) as u8);
        data_port.write(((divisor >> 8) & 0xFF) as u8);

        let status: u8 = speaker_port.read();
        if status != (status | 3) {
            speaker_port.write(status | 3);
        }
    }
}

/// Stops the PC speaker tone.
pub fn stop_tone() {
    let mut speaker_port = Port::new(SPEAKER_PORT);
    unsafe {
        let status: u8 = speaker_port.read();
        speaker_port.write(status & 0xFC);
    }
}

/// Play a "Success" chirp.
pub fn beep_success() {
    play_tone(880); // A5
                    // TODO: Implement non-blocking delay
}
