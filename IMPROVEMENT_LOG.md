# Improvement Log

This file tracks identified bottlenecks, defects, and proposed improvements for Phoenix OS.

## [Milestone 1] Project Architecture & JARVIS Vision

### Identified Potential Improvements
- **Automated Dependency Updates**: Set up Dependabot to keep Rust crates and GitHub Actions updated.
- **IPC Performance**: Research Shared Memory vs. Message Passing for the high-frequency JARVIS API.
- **Capability Schema**: Define a formal DSL or schema for capability tokens.

### Implemented Improvements
- **Modular Kernel**: Separated panic handler and test runner into dedicated modules.
- **Foundational Types**: Introduced `PhysAddr` and `VirtAddr` in `common`.
- **Developer Experience**: Added a `Makefile` for common tasks (build, test, clean, size).
- **Binary Monitoring**: Added `scripts/check_size.sh` to track kernel size growth.
- **Logging**: Integrated serial port logging for early boot diagnostics.
- **Vision Pivot**: Updated architecture to be JARVIS-aware (API-first, event-driven, capability-based).
