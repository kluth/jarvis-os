#!/bin/bash

# Start QEMU in the background
# We bind VNC to 0.0.0.0:0 (port 5900)
# Use a lock file to prevent multiple instances
LOCKFILE=/tmp/qemu.lock
if [ -f "$LOCKFILE" ]; then
    echo "QEMU already running or lock file exists."
    exit 1
fi
touch "$LOCKFILE"

qemu-system-x86_64 \
    -m 512 \
    -drive format=raw,file=/app/jarvis-os.img \
    -vnc 0.0.0.0:0 \
    -vga virtio \
    -net nic,model=virtio -net user \
    -serial mon:stdio \
    &

# Poll for QEMU readiness instead of fixed sleep
# QEMU binds to 5900 when ready
MAX_RETRIES=30
RETRIES=0
while ! nc -z localhost 5900; do
    sleep 0.5
    ((RETRIES++))
    if [ $RETRIES -eq $MAX_RETRIES ]; then
        echo "QEMU failed to start within timeout."
        rm "$LOCKFILE"
        exit 1
    fi
done

echo "QEMU is ready on port 5900."

# Start websockify to bridge VNC (5900) to WebSocket (8080)
websockify --web=/usr/share/novnc/ 8080 localhost:5900

# Cleanup on exit
rm "$LOCKFILE"
