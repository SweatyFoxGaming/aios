# Improvement Log

This file tracks identified bottlenecks, defects, and proposed improvements for Phoenix OS.

## [Milestone 1] Project Architecture

### Identified Potential Improvements
- **Automated Dependency Updates**: Set up Dependabot to keep Rust crates and GitHub Actions updated.
- **Cross-Compilation Setup**: Refine the build system for easier cross-compilation to `x86_64-unknown-none`. (Done)
- **Pre-commit Hooks**: Add a `pre-commit` configuration for local development.

### Implemented Improvements
- **Modular Kernel**: Separated panic handler and test runner into dedicated modules.
- **Foundational Types**: Introduced `PhysAddr` and `VirtAddr` in `common`.
- **Developer Experience**: Added a `Makefile` for common tasks (build, test, clean, size).
- **Binary Monitoring**: Added `scripts/check_size.sh` to track kernel size growth.
- **Build Configuration**: Fixed `no_std` kernel build by setting `panic = "abort"` and making `common` library's `no_std` conditional for host tests.
- **Linker Robustness**: Improved linker script with section wildcards for better symbol placement.
- **Workspace Integration**: Linked `kernel` and `common` crates properly.
