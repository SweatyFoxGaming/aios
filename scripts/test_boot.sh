#!/bin/bash
set -uo pipefail

# Formalizes the manual QEMU-boot-and-grep-log verification used throughout
# this project's history into a scripted, non-zero-exit-on-failure check of
# the production ISO. The kernel's main loop never returns, so `timeout`
# killing this process at TIMEOUT_SECS is expected -- only the log content
# is checked, not the process exit code.

ISO_NAME="${ISO_NAME:-phoenix-os-grub.iso}"
LOG_FILE="$(mktemp /tmp/phoenix_boot_XXXXXX.log)"
TIMEOUT_SECS="${TIMEOUT_SECS:-20}"
MARKER="[Lethe] Engine active."

if [ ! -f "$ISO_NAME" ]; then
    echo "ERROR: $ISO_NAME not found -- run scripts/build_iso.sh first." >&2
    exit 1
fi

timeout "$TIMEOUT_SECS" qemu-system-x86_64 \
    -cdrom "$ISO_NAME" \
    -serial file:"$LOG_FILE" \
    -display none \
    -no-reboot

echo "----- boot log ($LOG_FILE) -----"
cat "$LOG_FILE"
echo "---------------------------------"

if grep -iqE "exception|panic" "$LOG_FILE"; then
    echo "FAIL: boot log contains an exception/panic." >&2
    exit 1
fi
if ! grep -qF "$MARKER" "$LOG_FILE"; then
    echo "FAIL: boot log never reached '$MARKER'." >&2
    exit 1
fi
echo "PASS: clean boot, reached '$MARKER'."
