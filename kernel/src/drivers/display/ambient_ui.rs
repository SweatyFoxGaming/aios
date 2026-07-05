//! Ambient UI: the "Invisible Partner" idle/active state machine
//! (`docs/AMBIENT_UI_SPECIFICATION.md`) and the three functional panels
//! that materialize in the active state.

use crate::drivers::display::{Color, DISPLAY};
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

/// Whether the ambient UI is showing just the idle emblem, or the active
/// panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Idle,
    Active,
}

/// Which panel currently receives typed keyboard input. The Status panel
/// never takes focus (it has no input).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Command,
    Chat,
}

struct AmbientState {
    ui_state: UiState,
    focus: Focus,
}

static STATE: Mutex<AmbientState> = Mutex::new(AmbientState {
    ui_state: UiState::Idle,
    focus: Focus::Chat,
});

/// Text typed into the Command panel's input box, submitted to
/// `fs::shell::handle_command` on Enter. Separate from `keyboard.rs`'s
/// existing `INPUT_BUFFER`, which now serves only the Chat panel.
static COMMAND_INPUT: Mutex<String> = Mutex::new(String::new());

/// Command panel scrollback, capped at 20 lines -- when full, the oldest
/// line is removed before the newest is pushed, so this Vec's original
/// `with_capacity` allocation never needs to grow (growing an existing
/// heap allocation crashes on this target -- see safe_alloc.rs).
static COMMAND_SCROLLBACK: Mutex<Vec<String>> = Mutex::new(Vec::new());
const MAX_SCROLLBACK_LINES: usize = 20;

fn push_scrollback(buf: &Mutex<Vec<String>>, line: String) {
    let mut lines = buf.lock();
    if lines.capacity() == 0 {
        *lines = Vec::with_capacity(MAX_SCROLLBACK_LINES);
    }
    if lines.len() >= MAX_SCROLLBACK_LINES {
        lines.remove(0);
    }
    lines.push(line);
}

/// Called from `mouse.rs` on any movement or click.
pub fn on_mouse_activity() {
    let mut state = STATE.lock();
    if state.ui_state == UiState::Idle {
        state.ui_state = UiState::Active;
    }
    drop(state);
    handle_click_focus();
}

/// Called from `keyboard.rs` for every decoded key. `character` is `Some`
/// for a printable/Enter key, `None` alongside `escape: true` for the
/// Escape key, and `None`/`escape: false` for any other raw key (arrows,
/// function keys, etc. -- ignored for now).
pub fn on_key(character: Option<char>, escape: bool) {
    let mut state = STATE.lock();
    if escape {
        state.ui_state = UiState::Idle;
        return;
    }
    if state.ui_state == UiState::Idle {
        state.ui_state = UiState::Active;
    }
    let focus = state.focus;
    drop(state);

    let Some(ch) = character else { return };
    match focus {
        Focus::Command => command_panel_key(ch),
        Focus::Chat => crate::drivers::input::keyboard::chat_panel_key(ch),
    }
}

fn command_panel_key(ch: char) {
    if ch == '\n' {
        submit_command();
        return;
    }
    let mut buf = COMMAND_INPUT.lock();
    if buf.len() < 256 {
        buf.push(ch);
    }
}

/// Panel bounds for a 1024x768 framebuffer. See the design doc
/// (`docs/superpowers/specs/2026-07-05-ambient-ui-desktop-v1-design.md`)
/// section 4.2 for the layout rationale.
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect {
    const fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

const COMMAND_PANEL_RECT: Rect = Rect { x: 0, y: 0, w: 700, h: 384 };
const CHAT_PANEL_RECT: Rect = Rect { x: 0, y: 384, w: 700, h: 384 };

fn handle_click_focus() {
    let Some((cx, cy)) = crate::drivers::input::mouse::take_click() else {
        return;
    };
    let mut state = STATE.lock();
    if state.ui_state != UiState::Active {
        return;
    }
    if COMMAND_PANEL_RECT.contains(cx, cy) {
        state.focus = Focus::Command;
    } else if CHAT_PANEL_RECT.contains(cx, cy) {
        state.focus = Focus::Chat;
    }
}

/// Current UI state, read by `main.rs`'s render loop and the
/// panel-rendering code.
#[must_use]
pub fn current_state() -> UiState {
    STATE.lock().ui_state
}

/// Currently-focused panel, read by the panel-rendering code to draw a
/// focus border.
#[must_use]
pub fn current_focus() -> Focus {
    STATE.lock().focus
}

/// Submit the Command panel's current input to `fs::shell::handle_command`
/// and record both the command and its output in the scrollback.
pub fn submit_command() {
    let mut input = COMMAND_INPUT.lock();
    if input.is_empty() {
        return;
    }
    let cmd = crate::safe_alloc::to_string(&input);
    input.clear();
    drop(input);

    push_scrollback(&COMMAND_SCROLLBACK, crate::safe_alloc::concat2("> ", &cmd));
    let output = crate::fs::shell::handle_command(&cmd);
    if !output.is_empty() {
        push_scrollback(&COMMAND_SCROLLBACK, output);
    }
}

/// Temporary stub -- Task 8 replaces this with real Hermes dispatch and
/// chat scrollback rendering.
pub fn submit_chat_message() {}

/// Deep matte charcoal idle background, per
/// `docs/AMBIENT_UI_SPECIFICATION.md` section 2.
const IDLE_BACKGROUND: Color = Color { r: 10, g: 12, b: 16, a: 255 };

/// Initialize the Ambient UI -- called once from `main.rs` after the
/// framebuffer is set up.
pub fn init() {
    render();
}

/// Redraw the current state. Called in a loop from `main.rs`'s idle
/// spin -- v1 has no animation, so this simply redraws everything fresh
/// each call, which is cheap enough at 1024x768 for a polling loop.
///
/// Takes `&mut AuraDisplay` (not `&AuraDisplay`) even though `render_idle`
/// only needs `fill_circle`'s `&self`: Task 7 onward adds
/// `font::draw_text` calls to `render_active`, and `embedded-graphics`'s
/// `Drawable::draw` requires `&mut D: DrawTarget`.
pub fn render() {
    let mut guard = DISPLAY.lock();
    let Some(ref mut display) = *guard else {
        return;
    };
    display.clear(IDLE_BACKGROUND);
    match current_state() {
        UiState::Idle => render_idle(display),
        UiState::Active => render_active(display),
    }
}

fn render_idle(display: &mut crate::drivers::display::AuraDisplay) {
    // Centered geometric ring emblem -- matches the "Geometric Ring"
    // naming already used in the deleted aura.rs placeholder, now
    // actually drawn instead of just logged. `fill_circle` takes `&self`,
    // which auto-reborrows fine through the `&mut` reference.
    display.fill_circle(512, 384, 40, Color::CYAN);
}

fn render_active(display: &mut crate::drivers::display::AuraDisplay) {
    render_command_panel(display);
}

fn render_command_panel(display: &mut crate::drivers::display::AuraDisplay) {
    let border_color = if current_focus() == Focus::Command { Color::CYAN } else { Color::WHITE };
    display.draw_rect(0, 0, 700, 2, border_color);
    display.draw_rect(0, 0, 2, 384, border_color);
    display.draw_rect(698, 0, 2, 384, border_color);
    display.draw_rect(0, 382, 700, 2, border_color);

    let lines = COMMAND_SCROLLBACK.lock();
    let mut y = 10;
    for line in lines.iter() {
        crate::drivers::display::font::draw_text(display, 10, y, line, Color::WHITE);
        y += crate::drivers::display::font::LINE_HEIGHT;
    }
    drop(lines);
    let input = COMMAND_INPUT.lock();
    let prompt = crate::safe_alloc::concat2("> ", &input);
    crate::drivers::display::font::draw_text(display, 10, y, &prompt, Color::CYAN);
}
