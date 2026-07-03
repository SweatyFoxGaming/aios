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

/// Types of harmonic phrases.
pub enum HarmonicPhrase {
    /// A rising scale for starting a task.
    StartingTask,
    /// A descending scale for completing a task.
    TaskCompleted,
    /// A soft, rhythmic pulse for background processing.
    Thinking,
    /// A rapid, high-pitched chirp for alerts.
    Alert,
}

/// Plays a harmonic phrase.
pub fn play_phrase(phrase: HarmonicPhrase) {
    match phrase {
        HarmonicPhrase::StartingTask => {
            crate::println!("[Aether] Playing: Starting Task (Rising scale)");
            play_tone(440);
            play_tone(554);
            play_tone(659);
            stop_tone();
        }
        HarmonicPhrase::TaskCompleted => {
            crate::println!("[Aether] Playing: Task Completed (Falling scale)");
            play_tone(659);
            play_tone(554);
            play_tone(440);
            stop_tone();
        }
        HarmonicPhrase::Thinking => {
            crate::println!("[Aether] Playing: Thinking (Soft rhythmic pulse)");
            play_tone(220);
            stop_tone();
        }
        HarmonicPhrase::Alert => {
            crate::println!("[Aether] Playing: ALERT (Rapid chirp)");
            play_tone(880);
            play_tone(987);
            stop_tone();
        }
    }
}

/// Interpret a string into phonetic frequency shifts (Simulated Speech).
pub fn play_vocal_line(text: &str) {
    crate::println!("[Aether] Vocalizing: \"{}\"", text);
    for c in text.chars() {
        let freq = match c.to_ascii_lowercase() {
            'a' | 'e' | 'i' | 'o' | 'u' => 440, // Vowels are stable
            's' | 'f' | 'h' => 880,             // Sibilants are high
            'b' | 'd' | 'g' => 220,             // Plosives are low
            ' ' => 0,                           // Pause
            _ => 330,                           // Default
        };

        if freq > 0 {
            play_tone(freq);
            // In a real system, we'd have a small delay here
            stop_tone();
        }
    }
}

/// Play a "Success" chirp.
pub fn beep_success() {
    play_phrase(HarmonicPhrase::TaskCompleted);
}
