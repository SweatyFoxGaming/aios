# Phoenix OS - Common Library

The `common` library contains shared data structures, utilities, and logic used by both the kernel and userspace components of Phoenix OS.

## Constraints

- **`no_std`**: This library is strictly `no_std` to ensure compatibility with the kernel environment.
- **Efficiency**: Designed for low memory footprint and high performance.

## Current Status

- Basic project structure.
- Unit tests runnable on host.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
common = { path = "../common" }
```
