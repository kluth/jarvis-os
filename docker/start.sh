#!/bin/bash

# Start QEMU in the background
# We bind VNC to 0.0.0.0:0 (port 5900)
qemu-system-x86_64 \
    -m 512 \
    -drive format=raw,file=/app/jarvis-os.img \
    -vnc 0.0.0.0:0 \
    -vga virtio \
    -net nic,model=virtio -net user \
    -serial mon:stdio \
    &

# Wait for QEMU to start
sleep 2

# Start websockify to bridge VNC (5900) to WebSocket (8080)
# We use --vnc localhost:5900 since it's now bound to 0.0.0.0
websockify --web=/usr/share/novnc/ 8080 localhost:5900
