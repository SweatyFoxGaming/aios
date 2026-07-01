#!/bin/bash
# Phoenix OS Baseline Benchmark

echo "--- Phoenix OS Baseline Stats ---"
echo "Date: $(date)"

# Binary Size
KERNEL_BIN="target/x86_64-unknown-none/debug/kernel"
if [ -f "$KERNEL_BIN" ]; then
    SIZE=$(stat -c %s "$KERNEL_BIN")
    echo "Kernel Binary Size (Debug): $SIZE bytes"
fi

KERNEL_BIN_RELEASE="target/x86_64-unknown-none/release/kernel"
if [ -f "$KERNEL_BIN_RELEASE" ]; then
    SIZE_RELEASE=$(stat -c %s "$KERNEL_BIN_RELEASE")
    echo "Kernel Binary Size (Release): $SIZE_RELEASE bytes"
fi

# Build Time
echo "Measuring Build Time (Full Workspace)..."
cargo clean
START_TIME=$(date +%s)
cargo build --workspace > /dev/null 2>&1
END_TIME=$(date +%s)
echo "Build Time: $((END_TIME - START_TIME)) seconds"

echo "---------------------------------"
