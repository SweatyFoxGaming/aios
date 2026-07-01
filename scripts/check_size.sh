#!/bin/bash
set -e

KERNEL_BIN="target/x86_64-unknown-none/debug/kernel"

if [ ! -f "$KERNEL_BIN" ]; then
    echo "Kernel binary not found, building..."
    cargo build -p kernel
fi

SIZE=$(stat -c %s "$KERNEL_BIN")
echo "Kernel size: $SIZE bytes"

# Set a threshold for warning (e.g., 1MB for early kernel)
THRESHOLD=1048576
if [ "$SIZE" -gt "$THRESHOLD" ]; then
    echo "WARNING: Kernel size exceeds 1MB!"
fi
