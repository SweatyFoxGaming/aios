#!/bin/bash
set -uo pipefail

# Builds the kernel's #[test_case] harness, boots it headless in QEMU with
# the isa-debug-exit device, and turns the QEMU process exit code into
# pass/fail. Success (kernel/src/qemu_exit.rs QemuExitCode::Success = 0x10)
# maps to QEMU exit 33; Failed (0x11) maps to 35; anything else (including
# 124 from `timeout` killing a hang) is treated as a failure.

TIMEOUT_SECS="${TIMEOUT_SECS:-30}"
LOG_FILE="$(mktemp /tmp/phoenix_kerneltest_XXXXXX.log)"
BUILD_LOG="$(mktemp /tmp/phoenix_kerneltest_build_XXXXXX.log)"
TEST_ISO="phoenix-os-grub-test.iso"

echo "Building kernel test harness..."
EXECUTABLE=$(cargo +nightly test -p kernel --target x86_64-unknown-none \
    --no-run --message-format=json 2>"$BUILD_LOG" \
    | jq -r 'select(.executable != null) | .executable' | tail -n1)

if [ -z "$EXECUTABLE" ]; then
    echo "ERROR: could not locate kernel test executable." >&2
    cat "$BUILD_LOG" >&2
    exit 1
fi
echo "Test executable: $EXECUTABLE"

KERNEL_BIN="$EXECUTABLE" ISO_NAME="$TEST_ISO" ./scripts/build_iso.sh

timeout "$TIMEOUT_SECS" qemu-system-x86_64 \
    -cdrom "$TEST_ISO" \
    -serial file:"$LOG_FILE" \
    -display none \
    -no-reboot \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04
QEMU_EXIT=$?

echo "----- kernel test log ($LOG_FILE) -----"
cat "$LOG_FILE"
echo "----------------------------------------"

if [ "$QEMU_EXIT" -eq 33 ]; then
    echo "PASS: kernel test suite reported success."
    exit 0
fi
echo "FAIL: kernel test suite exited with code $QEMU_EXIT (expected 33)." >&2
exit 1
