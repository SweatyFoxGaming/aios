//! Aura: Framebuffer-based display driver for Phoenix OS.

pub mod aura;
pub mod engine;
pub mod transcendent;
use limine::Framebuffer;
use spin::Mutex;

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
    framebuffer: &'static Framebuffer,
    overlay_active: bool,
}

impl AuraDisplay {
    /// Create a new Aura display instance.
    #[must_use]
    pub const fn new(fb: &'static Framebuffer) -> Self {
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
        let ptr = self.framebuffer.address.as_ptr().unwrap();
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
        let ptr = self.framebuffer.address.as_ptr().unwrap();
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
}

/// Global access to the primary display.
pub static DISPLAY: Mutex<Option<AuraDisplay>> = Mutex::new(None);

/// Initialize the Aura display.
pub fn init(fb: &'static Framebuffer) {
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
