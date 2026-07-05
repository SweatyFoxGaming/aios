# Ambient UI Desktop v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a real, functional Ambient UI for Phoenix OS: a linear graphics framebuffer with basic drawing primitives and text rendering, real mouse/keyboard input, and three working panels (command shell, chat, system status) that materialize on input and dismiss on Escape.

**Architecture:** Fix the Multiboot2 boot handoff to request a real linear graphics framebuffer (currently misconfigured). Extend the existing `AuraDisplay` (`kernel/src/drivers/display/mod.rs`) with pixel/circle primitives and an `embedded-graphics` `DrawTarget` implementation for text rendering. Add a new `ambient_ui` module holding an `Idle`/`Active` state machine, panel layout, and focus routing, replacing the fully-fake `aura.rs`/`engine.rs`/`transcendent.rs` placeholder rendering (all three currently only `println!`, no real pixels are ever drawn). Wire real mouse position tracking and keyboard focus routing into the existing PS/2 interrupt handlers.

**Tech Stack:** Rust (`no_std`, `no_main`, x86_64-unknown-none, nightly, `-Z build-std=core,alloc`), `embedded-graphics = "0.8.2"` (verified compiling for this exact target with `default-features = false`) for bitmap-font text rendering, existing Multiboot2/GRUB2 boot chain.

## Global Constraints

These come from extensive, already-resolved toolchain/ABI bugs on this exact bare-metal target (see `kernel/src/safe_alloc.rs`'s module docs and `[[project_phoenix_os]]` memory for full history). **Every task below must follow these or it will reintroduce crashes that took a long debugging session to fix:**

- **Never call `.contains()`, `.split_whitespace()`, or `==`/`match` on `&str` values.** These use Rust's generic `Pattern`/`PartialEq` trait dispatch, which corrupts the return address once reached deep enough in a real boot. Use `crate::safe_alloc::contains`, `crate::safe_alloc::first_word`, and `crate::safe_alloc::str_eq` instead — all already implemented in `kernel/src/safe_alloc.rs`.
- **Never call `.to_string()`, `String::from()`, string `+` concatenation, or `alloc::format!`.** Use `crate::safe_alloc::to_string`, `crate::safe_alloc::concat2`/`concat3`, or the `crate::safe_format!` macro instead.
- **Never let a `Vec`/`String` grow past its original `Vec::with_capacity`/`String::with_capacity` allocation.** Growing an existing heap allocation crashes unpredictably. Every new collection in this plan is pre-sized and capped — when a task says "cap at N and drop the oldest," that means: if already at N, `remove(0)` before `push`, so length never exceeds N and the backing allocation never grows.
- **Never zero-initialize or bulk-copy a stack buffer above ~128 bytes** (e.g. `[0u8; 512]`, `Vec::resize`). Use `crate::safe_alloc::zeroed_vec`/`crate::safe_alloc::copy_from_slice`.
- **Never write more than one `println!` call in a function that also has a `&str`/`String` parameter it formats**, and never use a looped `{:.N}` precision float format (a single non-looped one is fine). Combine multi-line output into one `println!` with embedded `\n` instead.
- All of the above apply equally to any new function this plan adds, not just code that touches strings obviously — a match arm just printing a literal is fine; anything constructing/comparing/growing a `String`/`Vec` needs the safe_alloc equivalents.

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `kernel/boot32.asm` | Modify | Add Multiboot2 framebuffer request tag to the header |
| `kernel/Cargo.toml` | Modify | Add `embedded-graphics` dependency |
| `kernel/src/drivers/display/mod.rs` | Modify | Add `bpp` to `Framebuffer`; add `put_pixel`/`fill_circle` to `AuraDisplay`; implement `embedded_graphics` `DrawTarget`/`OriginDimensions` for `AuraDisplay`; drop `aura`/`engine`/`transcendent` submodule declarations, add `font`/`ambient_ui` |
| `kernel/src/drivers/display/font.rs` | Create | Thin wrapper over `embedded-graphics` mono-font text rendering |
| `kernel/src/drivers/display/ambient_ui.rs` | Create | State machine, panel layout, focus routing, the three panels' render + input logic |
| `kernel/src/drivers/display/aura.rs` | Delete | Fully fake (`draw_pixel` is a no-op `println!`-only stub); superseded by `mod.rs` primitives |
| `kernel/src/drivers/display/engine.rs` | Delete | Fully fake placeholder ring rendering; superseded by `ambient_ui.rs` |
| `kernel/src/drivers/display/transcendent.rs` | Delete | Fully fake placeholder ring rendering; superseded by `ambient_ui.rs` |
| `kernel/src/drivers/input/mouse.rs` | Modify | Add real cursor position/button-edge tracking, notify `ambient_ui` |
| `kernel/src/drivers/input/keyboard.rs` | Modify | Recognize Escape; route typed characters to whichever `ambient_ui` panel has focus |
| `kernel/src/fs/shell.rs` | Modify | `handle_command` returns its output as a `String` instead of only `println!`, so both the boot demo and the new command panel can use it |
| `kernel/src/pulse.rs` | Modify | Store last-computed pressure in a static; add `get_pressure()` |
| `kernel/src/vesta.rs` | Modify | Store last-computed consistency score in a static; add `get_consistency_score()` |
| `kernel/src/mnemosyne.rs` | Modify | Add `node_count()` |
| `kernel/src/main.rs` | Modify | Replace `engine::init()`/`transcendent::init()`/`aura::render_emblem()` calls with `ambient_ui::init()`; replace the final `loop {}` with one that calls `ambient_ui::render()` |

---

### Task 1: Real linear framebuffer

**Files:**
- Modify: `kernel/boot32.asm`
- Modify: `kernel/src/main.rs:195-201` (the `framebuffer_tag()` handling block)
- Modify: `kernel/src/drivers/display/mod.rs:13-22` (`Framebuffer` struct)

**Interfaces:**
- Produces: `Framebuffer { address: u64, pitch: u64, width: u64, height: u64, bpp: u8 }` (new `bpp` field) — every later task that draws reads this.

Currently GRUB hands back a `framebuffer_tag()` reporting 80×25 (VGA text-mode dimensions, not real pixels) because `boot32.asm`'s Multiboot2 header has no framebuffer request tag at all — just the magic/arch/size/checksum plus an empty end tag. This task adds an explicit request for a 1024×768×32 linear graphics mode.

- [ ] **Step 1: Add the framebuffer request tag to the Multiboot2 header**

Open `kernel/boot32.asm`. The current header is:

```asm
section .multiboot_header
header_start:
    dd MULTIBOOT2_MAGIC
    dd ARCH_I386
    dd header_end - header_start
    dd -(MULTIBOOT2_MAGIC + ARCH_I386 + (header_end - header_start))

    ; end tag
    dw 0
    dw 0
    dd 8
header_end:
```

Replace it with (inserting a framebuffer request tag, type 5, before the end tag — tag layout per the Multiboot2 spec: `u16 type; u16 flags; u32 size; u32 width; u32 height; u32 depth;`, padded to 8-byte alignment):

```asm
section .multiboot_header
header_start:
    dd MULTIBOOT2_MAGIC
    dd ARCH_I386
    dd header_end - header_start
    dd -(MULTIBOOT2_MAGIC + ARCH_I386 + (header_end - header_start))

    ; framebuffer request tag (type 5): ask for a 1024x768x32 linear
    ; graphics mode. GRUB may not honor this exactly (e.g. under UEFI it
    ; sometimes hands back no framebuffer tag at all) -- main.rs's
    ; existing `if let Some(Ok(fb_tag))` handling already tolerates that
    ; by simply not initializing the Ambient UI, which must stay true.
    ;
    ; `size` per the Multiboot2 spec covers the WHOLE tag, including this
    ; 8-byte type/flags/size header -- fb_tag_start must therefore start
    ; at the tag's first byte (`dw 5`), not after the size field, or the
    ; computed size undercounts by 8 and GRUB misparses the tag.
    align 8
fb_tag_start:
    dw 5                        ; type = framebuffer
    dw 0                        ; flags
    dd fb_tag_end - fb_tag_start ; size (20: 8-byte header + 3x4-byte fields)
    dd 1024                     ; width
    dd 768                      ; height
    dd 32                       ; depth (bits per pixel)
fb_tag_end:

    ; end tag
    align 8
    dw 0
    dw 0
    dd 8
header_end:
```

- [ ] **Step 2: Add `bpp` to the `Framebuffer` struct**

In `kernel/src/drivers/display/mod.rs`, change:

```rust
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
}
```

to:

```rust
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
    /// Bits per pixel (expected 32 after Task 1's Multiboot2 request;
    /// kept as a real field rather than assumed, since GRUB doesn't
    /// have to honor the requested depth).
    pub bpp: u8,
}
```

- [ ] **Step 3: Pass `bpp` through in `main.rs`**

In `kernel/src/main.rs`, change:

```rust
    if let Some(Ok(fb_tag)) = boot_info.framebuffer_tag() {
        let framebuffer = drivers::display::Framebuffer {
            address: fb_tag.address(),
            pitch: u64::from(fb_tag.pitch()),
            width: u64::from(fb_tag.width()),
            height: u64::from(fb_tag.height()),
        };
```

to:

```rust
    if let Some(Ok(fb_tag)) = boot_info.framebuffer_tag() {
        let framebuffer = drivers::display::Framebuffer {
            address: fb_tag.address(),
            pitch: u64::from(fb_tag.pitch()),
            width: u64::from(fb_tag.width()),
            height: u64::from(fb_tag.height()),
            bpp: fb_tag.bpp(),
        };
```

- [ ] **Step 4: Build and boot-test in QEMU with a real display**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none
./scripts/build_iso.sh
timeout 15 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot.log -no-reboot
```

Expected: a QEMU window opens (no `-display none` this time). Then check `/tmp/boot.log`:

```bash
grep "Framebuffer found" /tmp/boot.log
```

Expected: `Framebuffer found: 1024x768. Initializing Aura...` (not `80x25`). Also confirm the full boot log still reaches `[Lethe] Engine active.` with no `EXCEPTION`/panic, same as every prior verified run.

**Encountered when actually executing this task:** `drivers::display::init()` (called unconditionally as soon as a framebuffer tag exists, from the existing code in `main.rs`) immediately calls `AuraDisplay::clear()`, which writes to `self.framebuffer.address` directly as a raw pointer. QEMU's std VGA linear framebuffer sits at physical `0xfd000000` — outside the 2GiB that `boot32.asm`'s original page tables identity-map — so this produced `EXCEPTION: PAGE FAULT, Accessed Address: VirtAddr(0xfd000000)` immediately. Fixed by extending the identity mapping from 2GiB to 4GiB: `p2_tables` grows from `resb 4096 * 2` to `resb 4096 * 4`, `set_up_page_tables` maps `P3_low[0..4)` (not `[0..2)`) to all 4 P2 tables while leaving `P3_high` referencing only the first 2 (the higher-half/kernel mapping doesn't need to grow, only the identity side the framebuffer is accessed through), and the P2 fill loop covers 2048 entries (4GiB) instead of 1024. Verified this doesn't overflow 32-bit arithmetic in the `mov eax, 0x200000 / mul ecx` loop (max value at ecx=2047 is `0xffe00000`, still under `0xffffffff`). If implementing this plan on hardware/an emulator where the framebuffer address differs, adjust the mapped range accordingly — the key check is `grep "Accessed Address" /tmp/boot.log` after this step; if present, the mapping doesn't yet reach that address.

- [ ] **Step 5: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/boot32.asm kernel/src/main.rs kernel/src/drivers/display/mod.rs
git commit -m "feat(display): request a real 1024x768x32 linear framebuffer via Multiboot2"
```

---

### Task 2: Drawing primitives + embedded-graphics integration

**Files:**
- Modify: `kernel/Cargo.toml`
- Modify: `kernel/src/drivers/display/mod.rs`

**Interfaces:**
- Consumes: `Framebuffer` (Task 1, now with `bpp`).
- Produces: `AuraDisplay::put_pixel(&self, x: u64, y: u64, color: Color)`, `AuraDisplay::fill_circle(&self, cx: u64, cy: u64, radius: u64, color: Color)`, and `AuraDisplay` implementing `embedded_graphics::prelude::{OriginDimensions, DrawTarget<Color = embedded_graphics::pixelcolor::Rgb888, Error = core::convert::Infallible>}` — Task 3 (font) depends on this `DrawTarget` impl.

- [ ] **Step 1: Add the dependency**

In `kernel/Cargo.toml`, add one line under `[dependencies]` (order doesn't matter, but keep it alphabetically near the others for consistency):

```toml
embedded-graphics = { version = "0.8.2", default-features = false }
```

- [ ] **Step 2: Add `put_pixel` and `fill_circle` to `AuraDisplay`**

In `kernel/src/drivers/display/mod.rs`, add these methods to the existing `impl AuraDisplay` block (after `draw_rect`, before `toggle_overlay`):

```rust
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
```

- [ ] **Step 3: Implement `embedded_graphics::OriginDimensions` and `DrawTarget` for `AuraDisplay`**

Add near the top of `kernel/src/drivers/display/mod.rs` (after the existing `use spin::Mutex;`):

```rust
use embedded_graphics::{
    pixelcolor::{Rgb888, RgbColor},
    prelude::{DrawTarget, OriginDimensions, Point, Size},
    Pixel,
};
```

Add this impl block after `impl AuraDisplay { ... }` closes:

```rust
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
```

- [ ] **Step 4: Update module declarations**

In `kernel/src/drivers/display/mod.rs`, change:

```rust
pub mod aura;
pub mod engine;
pub mod transcendent;
```

to:

```rust
pub mod ambient_ui;
pub mod font;
```

(These two files don't exist yet — Tasks 3 and 5 create them. This will not compile until then; that's expected and fixed within this same task by adding minimal stub files.)

Create a temporary minimal `kernel/src/drivers/display/font.rs`:

```rust
//! Placeholder -- filled in by Task 3.
```

Create a temporary minimal `kernel/src/drivers/display/ambient_ui.rs`:

```rust
//! Placeholder -- filled in by Task 5.
```

- [ ] **Step 5: Delete the fully-fake placeholder files**

```bash
cd /mnt/ddrive/phoenix-os
rm kernel/src/drivers/display/aura.rs kernel/src/drivers/display/engine.rs kernel/src/drivers/display/transcendent.rs
```

- [ ] **Step 6: Fix the two remaining call sites that referenced the deleted modules**

`kernel/src/drivers/input/keyboard.rs` calls `crate::drivers::display::toggle_ghost_shell();` — this function lives in `mod.rs`, not the deleted files, so it's unaffected; leave it as-is.

`kernel/src/main.rs` currently has:

```rust
        drivers::display::init(framebuffer);
        drivers::display::engine::init();
        drivers::display::transcendent::init();
    }
```

and later:

```rust
    sched::process::load("Shell", alloc::vec![0x90, 0x90, 0x90]);
    drivers::display::aura::render_emblem();
    fs::shell::start();
```

Change the first block to:

```rust
        drivers::display::init(framebuffer);
    }
```

Leave the second block (`sched::process::load`, `render_emblem()`, `fs::shell::start()`) untouched for now — Task 10 replaces it once `ambient_ui` is fully built. For this task, just comment out the now-broken `render_emblem()` call so it compiles:

```rust
    sched::process::load("Shell", alloc::vec![0x90, 0x90, 0x90]);
    // drivers::display::aura::render_emblem(); // removed with aura.rs; ambient_ui::init() (Task 10) replaces this
    fs::shell::start();
```

- [ ] **Step 7: Build and verify**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
```

Expected: no errors. Then re-run the same boot test as Task 1 Step 4 to confirm the boot log still completes cleanly (deleting `aura.rs`/`engine.rs`/`transcendent.rs` removed their `println!` lines from the log — `[Aura] Ambient UI Engine online.` etc. will no longer appear, which is correct and expected).

- [ ] **Step 8: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add -A
git commit -m "feat(display): add real pixel/circle primitives and embedded-graphics DrawTarget, remove fake placeholder rendering"
```

---

### Task 3: Bitmap font rendering

**Files:**
- Modify: `kernel/src/drivers/display/font.rs` (replace placeholder from Task 2)

**Interfaces:**
- Consumes: `AuraDisplay` implementing `DrawTarget<Color = Rgb888>` (Task 2).
- Produces: `pub fn draw_text(display: &mut AuraDisplay, x: i32, y: i32, text: &str, color: Color)` — every panel-rendering task (7, 8, 9) depends on this.

- [ ] **Step 1: Write `font.rs`**

```rust
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
```

- [ ] **Step 2: Build**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
```

Expected: no errors. (No boot test yet — nothing calls `draw_text` until Task 6 onward.)

- [ ] **Step 3: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/drivers/display/font.rs
git commit -m "feat(display): add bitmap text rendering via embedded-graphics mono fonts"
```

---

### Task 4: Real mouse cursor tracking

**Files:**
- Modify: `kernel/src/drivers/input/mouse.rs`

**Interfaces:**
- Produces: `pub fn get_state() -> MouseState` (a copy of the current tracked position/buttons) and `pub fn take_click() -> Option<(i32, i32)>` (returns `Some((x, y))` exactly once per new left-click rising edge, `None` otherwise) — Task 5 (ambient_ui skeleton) and Task 6 (Idle/Active transition) depend on both.

The existing `handle_interrupt` decodes PS/2 packets into dx/dy deltas and button states but discards them entirely (`let _left = ...`). This task makes that real.

- [ ] **Step 1: Add tracked state**

In `kernel/src/drivers/input/mouse.rs`, add after the existing `MouseState` struct definition:

```rust
use lazy_static::lazy_static;
use spin::Mutex;

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
```

- [ ] **Step 2: Update `handle_interrupt` to store real state**

Replace the body of the `2 => { ... }` match arm (the final packet of each 3-byte PS/2 cycle):

```rust
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
```

Note: the clamp bounds (1023/767) match the 1024×768 framebuffer requested in Task 1. If that resolution ever changes, these must change with it — there's no dynamic lookup here since `mouse.rs` doesn't have access to the framebuffer struct; this is an acceptable, documented coupling for v1.

- [ ] **Step 3: Add the accessor functions**

At the end of `kernel/src/drivers/input/mouse.rs`:

```rust
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
```

`MouseState` needs `Clone`/`Copy` for the struct-literal copy above to be worth simplifying later, but a manual field-by-field copy (as written) works without adding derives, so no change to the struct definition is needed.

- [ ] **Step 4: Add a temporary stub so this compiles before Task 5**

This step calls `crate::drivers::display::ambient_ui::on_mouse_activity()`, which doesn't exist until Task 5. Add a minimal stub to the placeholder `ambient_ui.rs` from Task 2:

```rust
//! Placeholder -- filled in by Task 5.

/// Temporary no-op; Task 5 replaces this with real state-machine logic.
pub fn on_mouse_activity() {}
```

- [ ] **Step 5: Build and verify**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/drivers/input/mouse.rs kernel/src/drivers/display/ambient_ui.rs
git commit -m "feat(input): track real mouse cursor position and click edges"
```

---

### Task 5: Ambient UI state machine skeleton + keyboard focus routing

**Files:**
- Modify: `kernel/src/drivers/display/ambient_ui.rs` (replace Task 4's stub)
- Modify: `kernel/src/drivers/input/keyboard.rs`

**Interfaces:**
- Consumes: `crate::drivers::input::mouse::get_state()`/`take_click()` (Task 4), `crate::safe_alloc::to_string` (existing).
- Produces: `pub enum UiState { Idle, Active }`, `pub enum Focus { Command, Chat }`, `pub fn on_mouse_activity()` (replacing Task 4's stub), `pub fn on_key(character: Option<char>, escape: bool)`, `pub fn current_state() -> UiState`, `pub fn current_focus() -> Focus` — Tasks 6-9 (panels, rendering, main-loop wiring) all depend on this state.

This task wires the `Idle <-> Active` transition and keyboard/panel-focus plumbing, without yet drawing the panels themselves (Task 6 adds idle rendering; Tasks 7-9 add the panels; Task 10 wires it into `main.rs`'s loop).

- [ ] **Step 1: Write the state machine skeleton**

```rust
//! Ambient UI: the "Invisible Partner" idle/active state machine
//! (`docs/AMBIENT_UI_SPECIFICATION.md`) and the three functional panels
//! that materialize in the active state.

use alloc::string::String;
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
/// `fs::shell::handle_command` on Enter (Task 7). Separate from
/// `keyboard.rs`'s existing `INPUT_BUFFER`, which now serves only the
/// Chat panel.
static COMMAND_INPUT: Mutex<String> = Mutex::new(String::new());

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
        crate::drivers::display::ambient_ui::submit_command();
        return;
    }
    let mut buf = COMMAND_INPUT.lock();
    if buf.len() < 256 {
        buf.push(ch);
    }
}

/// Panel bounds for a 1024x768 framebuffer (Task 1). See the design doc
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

/// Current UI state, read by `main.rs`'s render loop (Task 10) and the
/// panel-rendering code (Tasks 6-9).
#[must_use]
pub fn current_state() -> UiState {
    STATE.lock().ui_state
}

/// Currently-focused panel, read by the panel-rendering code (Tasks 7-8)
/// to draw a focus border.
#[must_use]
pub fn current_focus() -> Focus {
    STATE.lock().focus
}
```

Note: `submit_command()` and `chat_panel_key()` are referenced here but implemented in Task 7 (command panel) and this same task's Step 2 (keyboard.rs) respectively — this file won't compile in isolation until Step 2 is also done; that's expected within this one task.

- [ ] **Step 2: Update `keyboard.rs` to route through the state machine**

In `kernel/src/drivers/input/keyboard.rs`, replace the `handle_interrupt` function body:

```rust
pub fn handle_interrupt() {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    if character == '`' {
                        // Toggle Ghost Shell with backtick (unrelated to
                        // the Ambient UI; kept as-is).
                        crate::drivers::display::toggle_ghost_shell();
                    } else {
                        crate::drivers::display::ambient_ui::on_key(Some(character), false);
                    }
                }
                DecodedKey::RawKey(pc_keyboard::KeyCode::Escape) => {
                    crate::drivers::display::ambient_ui::on_key(None, true);
                }
                DecodedKey::RawKey(_) => {}
            }
        }
    }
}
```

This removes the old always-on `INPUT_BUFFER`/`dispatch_buffer` wiring from `handle_interrupt` — Task 8 (Chat panel) reintroduces equivalent logic behind `chat_panel_key`, gated on Chat panel focus rather than always-on.

Replace the rest of the file (the `lazy_static!` block and `dispatch_buffer`) with:

```rust
lazy_static! {
    /// Global keyboard state.
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore
    ));

    /// Buffer for the Chat panel's input before Enter is pressed.
    /// Preallocated so `.push` below never needs to grow the buffer --
    /// growing an existing heap allocation crashes on this target (see
    /// safe_alloc.rs).
    static ref INPUT_BUFFER: Mutex<String> = Mutex::new(String::with_capacity(256));
}

/// Handle one character typed while the Chat panel has focus. Called from
/// `ambient_ui::on_key`. On Enter, dispatches the accumulated buffer to
/// the Hermes intent parser (Task 8 renders the result in the Chat
/// panel's scrollback).
pub fn chat_panel_key(character: char) {
    if character == '\n' {
        crate::drivers::display::ambient_ui::submit_chat_message();
    } else if INPUT_BUFFER.lock().len() < 256 {
        INPUT_BUFFER.lock().push(character);
    }
}

/// Take and clear the current Chat panel input buffer contents. Used by
/// `submit_chat_message` (Task 8).
pub fn take_chat_input() -> String {
    let mut buf = INPUT_BUFFER.lock();
    let taken = crate::safe_alloc::to_string(&buf);
    buf.clear();
    taken
}
```

`submit_chat_message()` is referenced here but implemented in Task 8 — again, this file compiles standalone only once that exists; expected within the overall Task 5+8 sequence, and Step 3 below adds a temporary stub so Task 5 alone still builds.

- [ ] **Step 3: Add temporary stubs for functions Tasks 7-8 will implement**

Add to the bottom of `ambient_ui.rs` (Tasks 7 and 8 will replace these with real logic):

```rust
/// Temporary stub -- Task 7 replaces this with real shell-command
/// dispatch and scrollback rendering.
pub fn submit_command() {}

/// Temporary stub -- Task 8 replaces this with real Hermes dispatch and
/// chat scrollback rendering.
pub fn submit_chat_message() {}
```

- [ ] **Step 4: Build and verify**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
```

Expected: no errors.

- [ ] **Step 5: Boot-test that Idle->Active state transition compiles and doesn't crash**

Nothing renders differently yet (Task 6 adds that), but this confirms the interrupt-handler changes haven't broken boot:

```bash
cd /mnt/ddrive/phoenix-os
./scripts/build_iso.sh
timeout 15 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot.log -no-reboot
grep -iE "exception|panic" /tmp/boot.log || echo "CLEAN"
```

Expected: `CLEAN`, and the full boot log through `[Lethe] Engine active.` as always.

- [ ] **Step 6: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/drivers/display/ambient_ui.rs kernel/src/drivers/input/keyboard.rs
git commit -m "feat(display): add Idle/Active state machine and keyboard focus routing"
```

---

### Task 6: Idle state rendering + main-loop wiring

**Files:**
- Modify: `kernel/src/drivers/display/ambient_ui.rs`
- Modify: `kernel/src/main.rs`

**Interfaces:**
- Consumes: `AuraDisplay::fill_circle`/`clear` (Task 2), `UiState`/`current_state()` (Task 5).
- Produces: `pub fn init()`, `pub fn render()` — Task 10 finalizes the `main.rs` wiring but this task does an initial version so the Idle emblem is visible and testable now, rather than waiting until every panel exists.

- [ ] **Step 1: Add `init()` and `render()` to `ambient_ui.rs`**

Add near the top of the file (after the existing `use` statements):

```rust
use crate::drivers::display::{Color, DISPLAY};
```

Add at the end of the file:

```rust
/// Deep matte charcoal idle background, per
/// `docs/AMBIENT_UI_SPECIFICATION.md` section 2.
const IDLE_BACKGROUND: Color = Color { r: 10, g: 12, b: 16, a: 255 };

/// Initialize the Ambient UI -- called once from `main.rs` after the
/// framebuffer is set up.
pub fn init() {
    render();
}

/// Redraw the current state. Called in a loop from `main.rs`'s idle
/// spin (Task 10) -- v1 has no animation, so this simply redraws
/// everything fresh each call, which is cheap enough at 1024x768 for a
/// polling loop.
///
/// Takes `&mut AuraDisplay` (not `&AuraDisplay`) even though `render_idle`
/// only needs `fill_circle`'s `&self`: Task 7 onward adds `font::draw_text`
/// calls to `render_active`, and `embedded-graphics`'s `Drawable::draw`
/// requires `&mut D: DrawTarget` -- deciding this now avoids reworking
/// every render function's signature partway through the plan.
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

fn render_active(_display: &mut crate::drivers::display::AuraDisplay) {
    // Panels added in Tasks 7-9.
}
```

- [ ] **Step 2: Wire `main.rs` to call `ambient_ui::init()` and loop on `render()`**

In `kernel/src/main.rs`, the framebuffer block currently reads (after Task 2's Step 6 edit):

```rust
        drivers::display::init(framebuffer);
    }
```

Change to:

```rust
        drivers::display::init(framebuffer);
        drivers::display::ambient_ui::init();
    }
```

And change the final block of the function from:

```rust
    #[cfg(test)]
    test_main();

    #[allow(clippy::empty_loop)]
    loop {}
}
```

to:

```rust
    #[cfg(test)]
    test_main();

    loop {
        drivers::display::ambient_ui::render();
    }
}
```

- [ ] **Step 3: Build and boot-test with a real QEMU display**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
./scripts/build_iso.sh
timeout 15 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot.log -no-reboot
```

Expected: the QEMU window shows a charcoal background with a cyan filled circle centered on screen. Then confirm the boot log is still clean:

```bash
grep -iE "exception|panic" /tmp/boot.log || echo "CLEAN"
```

**Encountered when actually executing this task:** a `screendump` taken via the QEMU monitor sometimes captured a fully blank (background-only) frame with no circle visible at all, on two separate attempts. This is not a rendering bug -- `fill_circle` was verified correct with a side-by-side diagnostic (a `draw_rect` square plus a `fill_circle`, both rendered correctly together). It's an artifact of this task's uncontrolled busy-loop redraw (`clear()` then redraw, as fast as possible, no vsync/double-buffering, explicitly deferred as "polish" per the design doc's Non-goals): `clear()` alone writes ~3MB (1024x768x32bpp) before the circle is drawn, so a screendump has a real chance of landing in that window. Not a blocker for this plan's scope; a future polish pass should double-buffer instead of clearing the live framebuffer directly.

- [ ] **Step 4: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/drivers/display/ambient_ui.rs kernel/src/main.rs
git commit -m "feat(display): render the idle emblem and wire the ambient UI into the main loop"
```

---

### Task 7: Command panel

**Files:**
- Modify: `kernel/src/fs/shell.rs`
- Modify: `kernel/src/drivers/display/ambient_ui.rs`

**Interfaces:**
- Consumes: `fs::shell::handle_command(&str) -> String` (modified return type), `font::draw_text` (Task 3), `AuraDisplay::draw_rect` (existing).
- Produces: nothing new consumed by later tasks (this is the last panel to depend on shell logic).

- [ ] **Step 1: Change `handle_command` to return its output as a `String`**

In `kernel/src/fs/shell.rs`, replace the whole function:

```rust
pub fn handle_command(cmd: &str) {
    let word = first_word(cmd);
    if word.is_empty() { return; }

    if str_eq(word, "help") {
        println!("Phoenix OS Shell\nAvailable commands: help, clear, info, ls, whoami, exit");
    } else if str_eq(word, "clear") {
        // In a real terminal, we would send ANSI escape codes
        println!("\x1B[2J\x1B[H");
    } else if str_eq(word, "info") {
        println!("Phoenix OS v0.1.0\nTarget: x86_64 Low-End Hardware\nStatus: Cognitive Core Active");
    } else if str_eq(word, "whoami") {
        println!("root@phoenix");
    } else if str_eq(word, "ls") {
        println!("Documents/\nSystem/\nPhoenixFS/");
    } else if str_eq(word, "exit") {
        println!("Shutting down shell...");
    } else {
        println!("Unknown command: {}", word);
    }
}
```

with:

```rust
pub fn handle_command(cmd: &str) -> String {
    let word = first_word(cmd);
    if word.is_empty() {
        return crate::safe_alloc::to_string("");
    }

    if str_eq(word, "help") {
        crate::safe_alloc::to_string("Available: help, clear, info, ls, whoami, exit")
    } else if str_eq(word, "clear") {
        crate::safe_alloc::to_string("")
    } else if str_eq(word, "info") {
        crate::safe_alloc::to_string("Phoenix OS v0.1.0 - Cognitive Core Active")
    } else if str_eq(word, "whoami") {
        crate::safe_alloc::to_string("root@phoenix")
    } else if str_eq(word, "ls") {
        crate::safe_alloc::to_string("Documents/ System/ PhoenixFS/")
    } else if str_eq(word, "exit") {
        crate::safe_alloc::to_string("Shutting down shell...")
    } else {
        crate::safe_alloc::concat2("Unknown command: ", word)
    }
}
```

Note this drops the multi-line `\n`-embedded strings from the old version — `safe_alloc::to_string`'s `MAX_LEN` is 128 bytes (see `kernel/src/safe_alloc.rs`), and keeping each reply as one short line avoids running close to that limit; the Command panel (Step 3 below) renders one scrollback entry per line anyway, so multi-line replies aren't needed.

`fs::shell::start()` (called once during boot, in `main.rs`, unrelated to this panel) calls `handle_command("info")` and `handle_command("ls")` and currently relies on it `println!`-ing directly. Update it to print the returned string:

```rust
/// Simulated shell loop.
pub fn start() {
    println!("--- Phoenix OS Native Shell ---\nType 'help' for a list of commands.");
    println!("phoenix> info");
    println!("{}", handle_command("info"));
    println!("phoenix> ls");
    println!("{}", handle_command("ls"));
}
```

- [ ] **Step 2: Add Command panel scrollback state and rendering to `ambient_ui.rs`**

Add near the top (with the other state):

```rust
use alloc::vec::Vec;

/// Command panel scrollback, capped at 20 lines -- when full, the oldest
/// line is removed before the newest is pushed, so this Vec's original
/// `with_capacity` allocation never needs to grow (see Global Constraints).
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
```

Replace the `submit_command` stub from Task 5:

```rust
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
```

Add Command panel rendering, replacing the empty `render_active` body from Task 6 (which already takes `&mut AuraDisplay` — see Task 6's note on why):

```rust
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
    let input = COMMAND_INPUT.lock();
    let prompt = crate::safe_alloc::concat2("> ", &input);
    crate::drivers::display::font::draw_text(display, 10, y, &prompt, Color::CYAN);
}
```

`AuraDisplay::draw_rect` (existing) takes `&self`, which is still valid to call through the `&mut AuraDisplay` parameter (Rust auto-reborrows), so no further signature changes are needed.

- [ ] **Step 3: Build and boot-test**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
./scripts/build_iso.sh
timeout 15 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot.log -no-reboot
```

Expected build: no errors. Manual verification in the QEMU window (this requires interactive input, which an automated agent can simulate via QEMU's monitor `sendkey` command, e.g. `sendkey a`, `sendkey ret`, to type "a" + Enter and confirm a Command panel border and typed prompt appear after any keypress switches from Idle to Active). At minimum, confirm the boot log stays clean:

```bash
grep -iE "exception|panic" /tmp/boot.log || echo "CLEAN"
```

- [ ] **Step 4: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/fs/shell.rs kernel/src/drivers/display/ambient_ui.rs
git commit -m "feat(display): add functional Command panel wired to fs::shell::handle_command"
```

---

### Task 8: Chat panel

**Files:**
- Modify: `kernel/src/drivers/display/ambient_ui.rs`

**Interfaces:**
- Consumes: `crate::hermes::parse`/`dispatch` (existing), `crate::drivers::input::keyboard::take_chat_input()` (Task 5), `font::draw_text` (Task 3).
- Produces: nothing new consumed by later tasks.

- [ ] **Step 1: Add Chat panel scrollback and canned-reply logic**

Add near `COMMAND_SCROLLBACK`:

```rust
static CHAT_SCROLLBACK: Mutex<Vec<String>> = Mutex::new(Vec::new());
```

Replace the `submit_chat_message` stub from Task 5:

```rust
/// Submit the Chat panel's current input to the existing Hermes intent
/// pipeline and record both the message and Phoenix's canned reply (keyed
/// off the classified intent action) in the scrollback. No real AI/LLM
/// response generation -- explicitly out of scope for v1 (see the design
/// doc's Non-goals).
pub fn submit_chat_message() {
    let message = crate::drivers::input::keyboard::take_chat_input();
    if message.is_empty() {
        return;
    }
    push_scrollback(&CHAT_SCROLLBACK, crate::safe_alloc::concat2("You: ", &message));

    let intent = crate::hermes::parse(&message);
    crate::hermes::dispatch(&intent);

    let reply = if crate::safe_alloc::str_eq(&intent.action, "KnowledgeQuery") {
        "Researching that for you..."
    } else if crate::safe_alloc::str_eq(&intent.action, "SelfRepair") {
        "Running diagnostics..."
    } else {
        "Understood."
    };
    push_scrollback(&CHAT_SCROLLBACK, crate::safe_alloc::concat2("Phoenix: ", reply));
}
```

- [ ] **Step 2: Add Chat panel rendering**

Update `render_active` (from Task 7) to also render the Chat panel:

```rust
fn render_active(display: &mut crate::drivers::display::AuraDisplay) {
    render_command_panel(display);
    render_chat_panel(display);
}

fn render_chat_panel(display: &mut crate::drivers::display::AuraDisplay) {
    let border_color = if current_focus() == Focus::Chat { Color::CYAN } else { Color::WHITE };
    display.draw_rect(0, 384, 700, 2, border_color);
    display.draw_rect(0, 384, 2, 384, border_color);
    display.draw_rect(698, 384, 2, 384, border_color);
    display.draw_rect(0, 766, 700, 2, border_color);

    let lines = CHAT_SCROLLBACK.lock();
    let mut y = 394;
    for line in lines.iter() {
        crate::drivers::display::font::draw_text(display, 10, y, line, Color::WHITE);
        y += crate::drivers::display::font::LINE_HEIGHT;
    }
}
```

- [ ] **Step 3: Build and boot-test**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
./scripts/build_iso.sh
timeout 15 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot.log -no-reboot
grep -iE "exception|panic" /tmp/boot.log || echo "CLEAN"
```

- [ ] **Step 4: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/drivers/display/ambient_ui.rs
git commit -m "feat(display): add functional Chat panel using the existing Hermes intent pipeline"
```

---

### Task 9: Status panel

**Files:**
- Modify: `kernel/src/pulse.rs`
- Modify: `kernel/src/vesta.rs`
- Modify: `kernel/src/mnemosyne.rs`
- Modify: `kernel/src/drivers/display/ambient_ui.rs`

**Interfaces:**
- Consumes: `ego::get_state()` (existing), new `pulse::get_pressure()`, `vesta::get_consistency_score()`, `mnemosyne::node_count()`.
- Produces: nothing new consumed by later tasks (last panel).

- [ ] **Step 1: Expose pulse pressure**

In `kernel/src/pulse.rs`, replace the whole file's relevant parts. Current:

```rust
/// Check current system pressure and emit events if thresholds are exceeded.
pub fn monitor() {
    // Placeholder: In a real system, we'd query the Frame Allocator and Scheduler
    let pressure = Pressure {
        memory: 0.2, // Simulated low pressure
        cpu: 0.1,
    };

    if pressure.memory > 0.8 {
```

Add a static above `monitor` and store into it:

```rust
use spin::Mutex;

static LAST_PRESSURE: Mutex<Pressure> = Mutex::new(Pressure { memory: 0.0, cpu: 0.0 });

/// Check current system pressure and emit events if thresholds are exceeded.
pub fn monitor() {
    // Placeholder: In a real system, we'd query the Frame Allocator and Scheduler
    let pressure = Pressure {
        memory: 0.2, // Simulated low pressure
        cpu: 0.1,
    };
    *LAST_PRESSURE.lock() = Pressure { memory: pressure.memory, cpu: pressure.cpu };

    if pressure.memory > 0.8 {
```

(`Pressure` has no `Copy`/`Clone` derive currently -- leave it that way and construct a new struct literal as shown, rather than adding derives not needed elsewhere.)

Add at the end of the file:

```rust
/// Last-computed system pressure, for the Ambient UI's status panel.
#[must_use]
pub fn get_pressure() -> Pressure {
    let p = LAST_PRESSURE.lock();
    Pressure { memory: p.memory, cpu: p.cpu }
}
```

- [ ] **Step 2: Expose vesta consistency score**

In `kernel/src/vesta.rs`, current:

```rust
/// Checks for inconsistencies or "exhaustion" in AI modules.
pub fn check_health() {
    // Placeholder for actual AI consistency checks
    let consistency_score = 1.0;

    if consistency_score < 0.7 {
```

Change to:

```rust
use spin::Mutex;

static LAST_SCORE: Mutex<f32> = Mutex::new(1.0);

/// Checks for inconsistencies or "exhaustion" in AI modules.
pub fn check_health() {
    // Placeholder for actual AI consistency checks
    let consistency_score = 1.0;
    *LAST_SCORE.lock() = consistency_score;

    if consistency_score < 0.7 {
```

Add at the end of the file:

```rust
/// Last-computed consistency score, for the Ambient UI's status panel.
#[must_use]
pub fn get_consistency_score() -> f32 {
    *LAST_SCORE.lock()
}
```

- [ ] **Step 3: Expose mnemosyne node count**

In `kernel/src/mnemosyne.rs`, add after `add_relation`:

```rust
/// Number of nodes currently in the knowledge graph, for the Ambient UI's
/// status panel.
#[must_use]
pub fn node_count() -> usize {
    GRAPH.lock().nodes.len()
}
```

- [ ] **Step 4: Add Status panel rendering to `ambient_ui.rs`**

Update `render_active` (from Task 8):

```rust
fn render_active(display: &mut crate::drivers::display::AuraDisplay) {
    render_command_panel(display);
    render_chat_panel(display);
    render_status_panel(display);
}

fn render_status_panel(display: &mut crate::drivers::display::AuraDisplay) {
    display.draw_rect(700, 0, 2, 768, Color::WHITE);

    let ego_state = crate::safe_format!("Presence: {:?}", crate::ego::get_state());
    let pressure = crate::pulse::get_pressure();
    let memory_line = crate::safe_format!("Memory pressure: {}", pressure.memory);
    let vesta_line = crate::safe_format!("Consistency: {}", crate::vesta::get_consistency_score());
    let nodes_line = crate::safe_format!("Knowledge nodes: {}", crate::mnemosyne::node_count());

    let lines = [&ego_state, &memory_line, &vesta_line, &nodes_line];
    let mut y = 10;
    for line in lines {
        crate::drivers::display::font::draw_text(display, 710, y, line, Color::WHITE);
        y += crate::drivers::display::font::LINE_HEIGHT;
    }
}
```

Note: `safe_format!` (not `alloc::format!`) is required per Global Constraints — it's already implemented in `kernel/src/safe_alloc.rs` and used elsewhere in the codebase (e.g. `ego.rs`'s `set_state`).

- [ ] **Step 5: Build and boot-test**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
./scripts/build_iso.sh
timeout 15 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot.log -no-reboot
grep -iE "exception|panic" /tmp/boot.log || echo "CLEAN"
```

- [ ] **Step 6: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/pulse.rs kernel/src/vesta.rs kernel/src/mnemosyne.rs kernel/src/drivers/display/ambient_ui.rs
git commit -m "feat(display): add functional Status panel reading real ego/pulse/vesta/mnemosyne state"
```

---

### Task 10: Final wiring and full regression test

**Files:**
- Modify: `kernel/src/main.rs`

**Interfaces:**
- Consumes: everything from Tasks 1-9.
- Produces: nothing further (final integration task).

- [ ] **Step 1: Remove the now-fully-superseded boot-demo calls**

In `kernel/src/main.rs`, the boot sequence still has (from Task 2 Step 6's temporary comment-out):

```rust
    sched::process::load("Shell", alloc::vec![0x90, 0x90, 0x90]);
    // drivers::display::aura::render_emblem(); // removed with aura.rs; ambient_ui::init() (Task 10) replaces this
    fs::shell::start();
```

`ambient_ui::init()` (Task 6) already renders the idle emblem for real, and `fs::shell::start()`'s hardcoded "info"/"ls" demo calls are no longer the primary way to use the shell (the real Command panel is, per Task 7). Simplify to just the process-load line and drop both now-redundant calls:

```rust
    sched::process::load("Shell", alloc::vec![0x90, 0x90, 0x90]);
```

`fs::shell::start` itself (the function definition in `shell.rs`) can stay as dead code for now — deleting it is optional cleanup, not required for this plan's scope, and leaving it avoids touching more of `shell.rs` than necessary in this task.

- [ ] **Step 2: Full build and regression test**

```bash
cd /mnt/ddrive/phoenix-os
RUSTUP_TOOLCHAIN=nightly cargo +nightly build -p kernel -Z build-std=core,alloc --target x86_64-unknown-none 2>&1 | grep -E "^error" -A15
./scripts/build_iso.sh
rm -f /tmp/boot_final.log
timeout 20 qemu-system-x86_64 -cdrom phoenix-os-grub.iso -serial file:/tmp/boot_final.log -no-reboot
```

Expected: QEMU window shows the charcoal background with the cyan idle emblem. Then:

```bash
grep -iE "exception|panic" /tmp/boot_final.log || echo "CLEAN"
tail -20 /tmp/boot_final.log
```

Expected: `CLEAN`, and the boot log ends at the same stable point as every prior verified run (no new crashes introduced by any of this work).

- [ ] **Step 3: Interactive verification checklist (manual or via QEMU monitor `sendkey`)**

- Press any key or move the mouse (in QEMU, click inside the window then use `sendkey`): background switches from idle emblem to the two-panel-plus-status Active layout.
- Type in the Command panel (default/last focus, or click its region first): characters appear after the `>` prompt; press Enter; the typed command and its `fs::shell::handle_command` output appear in scrollback (try `help`, `info`, `ls`, `whoami`, and an unknown command).
- Click inside the Chat panel region, type a message containing "research", press Enter: a "You: ..." line and a "Phoenix: Researching that for you..." line both appear.
- Confirm the Status panel shows real, changing values consistent with `ego`/`pulse`/`vesta`/`mnemosyne`'s actual current state (not hardcoded placeholder text).
- Press Escape: the view returns to the idle emblem.

- [ ] **Step 4: Real-hardware verification (once QEMU checks above all pass)**

Per `scripts/build_iso.sh` and `[[project_phoenix_os]]` memory: flash `phoenix-os-grub.iso` to the confirmed-working USB and boot via **Legacy/CSM mode** (UEFI hangs on this test machine independent of this work). Confirm the same checklist from Step 3 on real hardware, and confirm the on-screen graphics (not just text, unlike the earlier VGA-text-console milestone) render correctly on the physical display.

- [ ] **Step 5: Commit**

```bash
cd /mnt/ddrive/phoenix-os
git add kernel/src/main.rs
git commit -m "feat(display): finish wiring Ambient UI v1, remove superseded boot-demo calls"
git push origin milestone-1-architecture-16819735381189023400
```
