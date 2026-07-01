# Changelog

All notable changes to Phoenix OS will be documented in this file.

## [0.1.0] - Unreleased

### Added
- Initial project structure and repository architecture.
- Cargo workspace with `kernel` and `common` crates.
- GitHub Actions CI configuration for basic build and linting.
- Project documentation (README, ROADMAP, IMPROVEMENT_LOG).
- `no_std` kernel configuration for `x86_64-unknown-none`.
- Kernel-space testing framework skeleton.
- Foundational `PhysAddr` and `VirtAddr` types.
- `Makefile` for streamlined development.
- Kernel binary size monitoring script.
- Serial logging via COM1 for debugging.
- Initial Limine bootloader integration (Requests setup).
- Capability-based security and API-first core architecture vision.

### Changed
- Major architectural pivot: Phoenix OS is now an AI-Native OS centered around JARVIS.
- Refactored kernel `main.rs` to separate modules for panic and testing.
- Improved linker script robustness using wildcards.

### Fixed
- Fixed kernel build failure by adding `panic = "abort"` to workspace profiles.
- Fixed `common` crate test failure by making `#![no_std]` conditional.
