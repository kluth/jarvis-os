#!/usr/bin/env bash

# JARVIS OS Stability Watchdog
# Monitors QEMU output for Heartbeats and Panics.

TIMEOUT=${1:-300} # Default 300 seconds (5 minutes)
ISO_IMAGE="target/x86_64-jarvis_os/debug/bootimage-jarvis_os.iso"

if [ ! -f "$ISO_IMAGE" ]; then
    echo "Error: ISO image not found at $ISO_IMAGE"
    exit 1
fi

echo "Starting stability test (timeout: ${TIMEOUT}s)..."

# Run QEMU in the background, redirecting serial to a pipe
# -display none: we only care about serial output
# -device isa-debug-exit: allowed to exit if needed (though we'll kill it)
# -serial stdio: output heartbeats to stdout
qemu-system-x86_64 \
    -drive format=raw,file="$ISO_IMAGE" \
    -display none \
    -serial stdio \
    -m 512 \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    2>&1 | tee stability.log &

QEMU_PID=$!
START_TIME=$(date +%s)
HAS_PANIC=0
MAX_UPTIME=0

# Monitor output
while kill -0 $QEMU_PID 2>/dev/null; do
    # Check for Panic
    if grep -q "\[STABILITY_CHECK:PANIC\]" stability.log; then
        echo "FAIL: Kernel Panic detected!"
        HAS_PANIC=1
        kill $QEMU_PID
        break
    fi

    # Extract Max Uptime from Heartbeats
    UPTIME=$(grep "\[STABILITY_CHECK:HEARTBEAT\]" stability.log | tail -n 1 | sed -n 's/.*uptime=\([0-9]*\)s/\1/p')
    if [ ! -z "$UPTIME" ]; then
        MAX_UPTIME=$UPTIME
    fi

    # Check Timeout
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    if [ $ELAPSED -ge $TIMEOUT ]; then
        echo "SUCCESS: Stability threshold reached (${ELAPSED}s)."
        kill $QEMU_PID
        break
    fi

    sleep 1
done

echo "--- Stability Test Report ---"
echo "Duration: $(( $(date +%s) - START_TIME ))s"
echo "Max Reported Uptime: ${MAX_UPTIME}s"
echo "Status: $( [ $HAS_PANIC -eq 0 ] && echo "PASSED" || echo "FAILED" )"

if [ $HAS_PANIC -eq 1 ]; then
    exit 1
fi
exit 0
