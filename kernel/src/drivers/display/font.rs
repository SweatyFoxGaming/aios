//! Bitmap text rendering for the Ambient UI, built on `embedded-graphics`'s
//! mono-font support rather than hand-authored glyph data -- a hand-rolled
//! bitmap font risks subtly wrong glyphs with no easy way to verify
//! correctness on a bare-metal target with no reference rendering to
//! compare against. `embedded-graphics` is a well-tested, widely-used
//! `no_std` crate (verified building for this exact x86_64-unknown-none
//! target with `default-features = false`).

use super::{AuraDisplay, Color};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::Point,
    text::Text,
    Drawable,
};

/// Draw a single line of text at `(x, y)` (top-left corner of the first
/// glyph). Multi-line text is not supported here -- callers split text
/// into lines themselves and call this once per line (see the panel
/// rendering in `ambient_ui.rs`).
pub fn draw_text(display: &mut AuraDisplay, x: i32, y: i32, text: &str, color: Color) {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(color.r, color.g, color.b));
    // `Text::draw` returns `Result<Point, Infallible>` (the position after
    // the drawn text) -- `AuraDisplay::draw_iter` never actually fails, so
    // this can't return an error in practice.
    let _ = Text::new(text, Point::new(x, y), style).draw(display);
}

/// Height in pixels of one line of `FONT_6X10` text, for callers laying
/// out multiple lines (panel scrollback rendering).
pub const LINE_HEIGHT: i32 = 10;
