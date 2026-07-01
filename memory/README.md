# Phoenix OS - Memory Management

Responsible for managing physical and virtual memory.

## Components

- **Physical Memory Manager**: Tracks free and used physical frames (e.g., Bitmap or Buddy allocator).
- **Virtual Memory Manager**: Handles page tables and address space mapping.
- **Heap Allocator**: Provides dynamic memory allocation (`alloc` crate integration).

## Goals

- Efficient memory usage.
- Support for 1 GB minimum RAM target.
- Safety and isolation.
