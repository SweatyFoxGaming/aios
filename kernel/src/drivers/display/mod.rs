//! Aura: Framebuffer-based display driver for Phoenix OS.

pub mod ambient_ui;
pub mod font;
use embedded_graphics::{
    pixelcolor::{Rgb888, RgbColor},
    prelude::{DrawTarget, OriginDimensions, Size},
    Pixel,
};
use spin::Mutex;

/// Framebuffer geometry/location, decoupled from any specific boot protocol
/// (previously tied directly to `limine::Framebuffer`) so display code
/// doesn't care whether the info came from Limine, Multiboot2, or anything
/// else that can hand us a linear framebuffer address.
#[derive(Debug, Clone, Copy)]
pub struct Framebuffer {
    /// Physical/mapped address of the linear framebuffer.
    pub address: u64,
    /// Bytes per scanline.
    pub pitch: u64,
    /// Width in pixels.
    pub width: u64,
    /// Height in pixels.
    pub height: u64,
    /// Bits per pixel (expected 32 after requesting a linear graphics
    /// mode via the Multiboot2 header; kept as a real field rather than
    /// assumed, since GRUB doesn't have to honor the requested depth).
    pub bpp: u8,
}

/// A simple RGBA color.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha component.
    pub a: u8,
}

impl Color {
    /// Deep charcoal black.
    pub const BLACK: Self = Self {
        r: 10,
        g: 12,
        b: 16,
        a: 255,
    }; // #0A0C10
    /// Pure white.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Muted cyan accent.
    pub const CYAN: Self = Self {
        r: 0,
        g: 188,
        b: 212,
        a: 255,
    };

    /// Pack color into a `u32` for the framebuffer.
    #[must_use]
    pub const fn pack(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Handle for the Aura display system.
pub struct AuraDisplay {
    framebuffer: Framebuffer,
    overlay_active: bool,
}

impl AuraDisplay {
    /// Create a new Aura display instance.
    #[must_use]
    pub const fn new(fb: Framebuffer) -> Self {
        Self {
            framebuffer: fb,
            overlay_active: false,
        }
    }

    /// Clear the screen with a specific color.
    ///
    /// # Panics
    /// Panics if the framebuffer address is invalid or if target size overflows.
    pub fn clear(&self, color: Color) {
        let ptr = self.framebuffer.address as *mut u8;
        let size = (self.framebuffer.pitch * self.framebuffer.height) / 4;
        let raw_color = color.pack();

        unsafe {
            for i in 0..size {
                #[allow(clippy::cast_ptr_alignment)]
                ptr.cast::<u32>()
                    .add(usize::try_from(i).unwrap())
                    .write_volatile(raw_color);
            }
        }
    }

    /// Draw a simple rectangle (placeholder for Phoenix Emblem).
    ///
    /// # Panics
    /// Panics if the framebuffer address is invalid or if coordinate calculations overflow.
    pub fn draw_rect(&self, x: u64, y: u64, width: u64, height: u64, color: Color) {
        let ptr = self.framebuffer.address as *mut u8;
        let pitch = self.framebuffer.pitch / 4;
        let raw_color = color.pack();

        for curr_y in y..(y + height) {
            for curr_x in x..(x + width) {
                unsafe {
                    #[allow(clippy::cast_ptr_alignment)]
                    ptr.cast::<u32>()
                        .add(usize::try_from(curr_y * pitch + curr_x).unwrap())
                        .write_volatile(raw_color);
                }
            }
        }
    }

    /// Toggles the "Ghost" overlay.
    pub fn toggle_overlay(&mut self) {
        self.overlay_active = !self.overlay_active;
        if self.overlay_active {
            // Render the ghost overlay border
            self.draw_rect(0, 0, self.framebuffer.width, 5, Color::CYAN);
            crate::println!("[Aura] Ghost Shell Overlay Activated.");
        } else {
            // Remove border
            self.draw_rect(0, 0, self.framebuffer.width, 5, Color::BLACK);
            crate::println!("[Aura] Ghost Shell Overlay Deactivated.");
        }
    }

    /// Write a single pixel. Bounds-checked; out-of-range coordinates are
    /// silently ignored rather than panicking, since callers (embedded-
    /// graphics text rendering, circle drawing) routinely compute
    /// coordinates that fall outside the screen at the edges.
    pub fn put_pixel(&self, x: u64, y: u64, color: Color) {
        if x >= self.framebuffer.width || y >= self.framebuffer.height {
            return;
        }
        let ptr = self.framebuffer.address as *mut u8;
        let pitch = self.framebuffer.pitch / 4;
        let raw_color = color.pack();
        unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            ptr.cast::<u32>()
                .add(usize::try_from(y * pitch + x).unwrap())
                .write_volatile(raw_color);
        }
    }

    /// Fill a horizontal span of pixels from `x0` to `x1` inclusive (helper
    /// for `fill_circle`). Negative coordinates are clamped to 0 rather
    /// than skipped, matching `put_pixel`'s own bounds handling.
    fn fill_span(&self, x0: i64, x1: i64, y: i64, color: Color) {
        if y < 0 {
            return;
        }
        let start = x0.max(0);
        let mut x = start;
        while x <= x1 {
            #[allow(clippy::cast_sign_loss)]
            self.put_pixel(x as u64, y as u64, color);
            x += 1;
        }
    }

    /// Fill a filled circle using the integer-only Bresenham/midpoint
    /// circle algorithm (no floating point -- this `no_std` kernel has no
    /// `libm`, and `core::f64` has no `sqrt` without it), drawing
    /// horizontal spans across the 4 symmetric octant pairs per step
    /// rather than individual points.
    pub fn fill_circle(&self, cx: u64, cy: u64, radius: u64, color: Color) {
        let cx = i64::try_from(cx).unwrap();
        let cy = i64::try_from(cy).unwrap();
        let mut x = i64::try_from(radius).unwrap();
        let mut y: i64 = 0;
        let mut err: i64 = 0;

        while x >= y {
            self.fill_span(cx - x, cx + x, cy + y, color);
            self.fill_span(cx - x, cx + x, cy - y, color);
            self.fill_span(cx - y, cx + y, cy + x, color);
            self.fill_span(cx - y, cx + y, cy - x, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }
}

impl OriginDimensions for AuraDisplay {
    fn size(&self) -> Size {
        Size::new(
            u32::try_from(self.framebuffer.width).unwrap(),
            u32::try_from(self.framebuffer.height).unwrap(),
        )
    }
}

impl DrawTarget for AuraDisplay {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x < 0 || point.y < 0 {
                continue;
            }
            #[allow(clippy::cast_sign_loss)]
            self.put_pixel(
                point.x as u64,
                point.y as u64,
                Color {
                    r: color.r(),
                    g: color.g(),
                    b: color.b(),
                    a: 255,
                },
            );
        }
        Ok(())
    }
}

/// Global access to the primary display.
pub static DISPLAY: Mutex<Option<AuraDisplay>> = Mutex::new(None);

/// Initialize the Aura display.
pub fn init(fb: Framebuffer) {
    let aura = AuraDisplay::new(fb);
    aura.clear(Color::BLACK);

    // Draw the "Phoenix Emblem" (minimalist representation: a cyan square in the center)
    let center_x = fb.width / 2;
    let center_y = fb.height / 2;
    aura.draw_rect(center_x - 10, center_y - 10, 20, 20, Color::CYAN);

    *DISPLAY.lock() = Some(aura);
}

/// Helper to toggle the ghost shell from other modules.
pub fn toggle_ghost_shell() {
    if let Some(ref mut aura) = *DISPLAY.lock() {
        aura.toggle_overlay();
    }
}
