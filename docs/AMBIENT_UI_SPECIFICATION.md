# Phoenix OS - Ambient UI Specification

This document defines the graphical interface philosophy and design language for Phoenix OS, centered around the **Ambient Computing** model.

## 1. Philosophy: "Nothing unless necessary"

The Phoenix OS interface is designed to disappear when not in use. It rejects the traditional desktop metaphor (icons, docks, taskbars) in favor of a calm, architectural space that responds only to user intent.

### Core Principles
- **Silence**: No persistent visual noise, notifications, or status monitors.
- **Confidence**: The system does not need to prove its intelligence through flashy animations.
- **Presence**: A waiting, patient atmosphere rather than a demanding one.
- **Intent-Driven**: The interface materializes as a specific workspace for a specific task and recedes immediately upon completion.

## 2. The Idle State: The Command Space

When idle, the screen is a deep charcoal black (#0A0C10) canvas with subtle gradients.

### The JARVIS Ring
At the exact center of the display is a single geometric form representing JARVIS:
- **Form**: Thin concentric rings with matte white highlights.
- **Accents**: Minimal muted cyan.
- **Behavior**: Perfectly still. No pulsing, breathing, or rotation.
- **Interaction**: Responds to the "Jarvis..." wake word with a faint increase in brightness and a subtle expansion of the rings. No voice visualizers or waveforms.

## 3. The Active State: Materializing Workspaces

The workspace is built dynamically around the user's current goal (e.g., "Build a Rust game").

- **Materialization**: Panels slide smoothly from the edges only as required by the current task.
- **Composition**: Empty space dominates. High-end spacing inspired by Apple Vision Pro.
- **Purpose**: Every visible element must have a direct relation to the expressed intent.
- **Recession**: Once the task is dismissed, the interface gracefully disappears, returning the system to the silent Command Space.

## 4. Design Language & Aesthetics

- **Inspiration**: Braun industrial design (Dieter Rams), Nothing Phone restraint, Japanese and Scandinavian minimalism.
- **Typography**: Thin, modern sans-serif with generous whitespace and large margins.
- **Color Palette**:
    - Primary: Deep Charcoal Black (#0A0C10)
    - Secondary: Graphite
    - Highlights: Soft White
    - Accents: Muted Cyan (No neon, no RGB)

## 5. Technical Implementation Goals

- **Latency**: Sub-millisecond response to intent expression.
- **Modular Panels**: UI components are registered as services via the Phoenix Service Bus.
- **Resource Aware**: UI rendering scales based on hardware; low-end systems maintain the same aesthetic with simplified geometry.
