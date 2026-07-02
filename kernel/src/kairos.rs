//! Kairòs: Context engine and situational awareness for Phoenix OS.

use crate::println;
use spin::Mutex;

/// Current system context.
pub struct Context {
    pub attention_point: (usize, usize),
    pub system_mood: f32, // 0.0 to 1.0 (Calm to Alert)
}

static CONTEXT: Mutex<Context> = Mutex::new(Context {
    attention_point: (400, 300),
    system_mood: 0.2,
});

/// Get the current user attention point (predicted focus).
pub fn get_attention_point() -> (usize, usize) {
    let ctx = CONTEXT.lock();
    ctx.attention_point
}

/// Set the current system attention point.
pub fn set_attention_point(x: usize, y: usize) {
    let mut ctx = CONTEXT.lock();
    ctx.attention_point = (x, y);
}

/// Log situational awareness status.
pub fn log_status() {
    let ctx = CONTEXT.lock();
    println!("[Kairòs] Context active. Mood: {:.2}, Focus: {:?}",
        ctx.system_mood, ctx.attention_point);
}
