# Ambient UI Desktop v1 — Design

**Date:** 2026-07-05
**Status:** Approved for implementation planning
**Roadmap item:** Phase 7 — "Ambient UI (Geometric Ring Materialization)" (`roadmap/ROADMAP.md`)

## 1. Background & scope

`docs/AMBIENT_UI_SPECIFICATION.md` already defines Phoenix OS's UI philosophy: an "Invisible Partner" model that explicitly rejects the traditional desktop metaphor (icons, docks, taskbars, windows). The idle state is a silent, centered emblem; the active state materializes only the tools a task needs, then fades away.

This spec covers **v1** of that Ambient UI, scoped to be genuinely functional rather than a visual mockup:

- A real linear graphics framebuffer (the current setup misreads VGA text-mode dimensions as pixel dimensions and is not usable for real drawing).
- Basic drawing primitives and an embedded bitmap font — enough to render the emblem, panels, and text, with no polish (no alpha blending, no rounded corners, no animated transitions).
- Real mouse cursor tracking (currently parsed and discarded) and keyboard focus routing.
- Three functional panels: a real command shell, a chat panel using the existing intent-parsing pipeline, and a system status readout.

**Non-goals for v1** (explicitly deferred to a later "polish" phase):
- Fade transitions, rounded corners, alpha blending, animation easing.
- Real AI/LLM-generated chat responses (chat panel uses the existing scripted-intent classification only).
- Voice or true "intent" detection — materialization is keyboard/mouse triggered.
- Multiple/resizable windows, docks, or any traditional windowing metaphor (explicitly out of scope per the Ambient UI philosophy).

## 2. Graphics foundation

### 2.1 Framebuffer fix
`kernel/boot32.asm`'s Multiboot2 header currently has no framebuffer request tag, and GRUB was observed handing back a `framebuffer_tag` reporting 80×25 — VGA text-mode dimensions, not real pixel dimensions — which `drivers/display/mod.rs`'s `AuraDisplay` currently misinterprets as a 4-bytes-per-pixel graphics buffer.

Fix: add an explicit Multiboot2 framebuffer request tag to the header, requesting a linear graphics mode (1024×768, 32 bpp preferred). `main.rs`'s existing `if let Some(Ok(fb_tag)) = boot_info.framebuffer_tag()` handling stays as the gate — if GRUB can't satisfy the request (as seen under UEFI, which sometimes hands back no framebuffer tag at all), Aura/the Ambient UI simply doesn't initialize, and the kernel continues exactly as it does today (this must not become a new boot blocker).

### 2.2 Drawing primitives
Extend `AuraDisplay` (`drivers/display/mod.rs`) with:
- `put_pixel(x, y, color)` — single-pixel write, foundation for everything else.
- `fill_circle(cx, cy, radius, color)` — for the emblem (matches the "Geometric Ring" naming already used in `drivers/display/aura.rs`'s placeholder rendering).
- Existing `fill_rect`/`clear` are reused as-is for panel backgrounds.

No alpha blending, no anti-aliasing — flat colors, hard edges, consistent with the "functional first" scope.

### 2.3 Bitmap font
Embed a small, standard 8×16 public-domain bitmap font (e.g. the classic IBM VGA font) as a static byte array. Add `draw_char(x, y, char, color)` and `draw_text(x, y, &str, color)` built on `put_pixel`. This is the only text rendering path for the Ambient UI (distinct from and independent of the existing serial/VGA-text-mode console used for boot logging, which is unaffected by this work).

## 3. Input system

### 3.1 Mouse
`kernel/src/drivers/input/mouse.rs`'s `handle_interrupt` currently decodes PS/2 packets into dx/dy deltas and button states but discards them (`let _left = ...`). Add:
- A global `MouseState { x: i32, y: i32, left: bool, right: bool, middle: bool }` behind a `Mutex`, matching the struct already defined but unused in that file.
- Accumulate dx/dy into `x`/`y`, clamped to `[0, framebuffer.width)` / `[0, framebuffer.height)`.
- Any movement or click, while in `Idle` state, triggers a transition to `Active`.

### 3.2 Keyboard
`kernel/src/drivers/input/keyboard.rs`'s `handle_interrupt` currently always accumulates typed characters into one global `INPUT_BUFFER` and dispatches to `hermes::parse`/`dispatch` on Enter. Changes:
- Recognize Escape (currently unhandled — falls into the `DecodedKey::RawKey` branch, which just logs it) and, when in `Active` state, trigger a transition back to `Idle`.
- Any other keypress, while in `Idle`, triggers a transition to `Active` (same as mouse).
- Route typed characters to whichever panel currently holds input focus (see 4.2) instead of always targeting the one global buffer — the Chat panel's input is the existing `INPUT_BUFFER`/hermes pipeline; the Command panel gets its own, separate input buffer.

## 4. State machine & layout

### 4.1 States
```
Idle   — charcoal (#0A0C10) background, centered emblem (fill_circle), no panels.
Active — same background, three panels visible (4.2).
```
Transitions: `Idle -> Active` on any keypress or mouse movement/click. `Active -> Idle` on Escape only (no auto-timeout in v1). No animated transition — instant redraw of the new state.

### 4.2 Layout (Active)
Two-column layout across the framebuffer:
- **Left column**, split horizontally: **Command panel** (top half), **Chat panel** (bottom half).
- **Right column**, full height: **Status panel**.

One panel holds input focus at a time (Command or Chat; Status never takes focus). Clicking inside a panel's bounds (using the mouse position from 3.1) or pressing Tab moves focus. The focused panel is visually indicated by a border drawn in the muted cyan already specified in `AMBIENT_UI_SPECIFICATION.md` (`Color::CYAN`, `#00BCD4`, already defined in `drivers/display/mod.rs`); unfocused panels get a plain white (`Color::WHITE`) border.

## 5. The three panels

### 5.1 Command panel
Real functionality: its input buffer's content, on Enter, is passed to the existing `fs::shell::handle_command(&str)` (`kernel/src/fs/shell.rs`) — unmodified. The panel keeps a scrollback of the last N lines of command + output text, rendered via `draw_text`, replacing what currently only goes to the serial/VGA console when this panel is focused and in use.

### 5.2 Chat panel
Reuses the existing keyboard buffer → `hermes::parse` → `hermes::dispatch` pipeline (`kernel/src/drivers/input/keyboard.rs`'s `dispatch_buffer`, `kernel/src/hermes.rs`) as-is. Renders a scrolling history of alternating "You: <message>" / "Phoenix: <reply>" lines. Phoenix's reply is a canned string selected by the parsed `Intent.action`:
- `KnowledgeQuery` → e.g. "Researching that for you..."
- `SelfRepair` → e.g. "Running diagnostics..."
- `GeneralInteraction` → e.g. "Understood."

No real AI-generated content — this is explicitly scripted per the approved design (see Non-goals).

### 5.3 Status panel
No input focus. On each materialization (transition to `Active`) and periodically while active, reads and renders as labeled text:
- `ego::get_state()` (Presence State)
- Pulse pressure (from `pulse::monitor`'s existing simulated values, or real values if pulse gains real metrics before this ships)
- Vesta health (`vesta::check_health`'s consistency score)
- Mnemosyne node count (from the existing knowledge graph)

All of this data already exists and is currently only exposed via the text-dump `log_status`/`debug_graph` functions called at the end of boot — this panel is a visual read-out of the same underlying state, not new data collection.

## 6. Testing

- QEMU with an actual display window (not `-display none`) at each implementation milestone, to visually verify rendering, cursor tracking, and panel layout.
- Boot-log regression check after the Multiboot2 header change (2.1) — confirm the existing full boot sequence (every subsystem through to stable idle) still completes cleanly, since this touches `boot32.asm`.
- Periodic real-hardware verification via the confirmed-working Legacy/CSM boot path (see `scripts/build_iso.sh` and `[[project_phoenix_os]]` memory) once the on-screen rendering is visually verified in QEMU.
