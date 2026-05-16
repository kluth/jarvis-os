#!/bin/bash

# Start QEMU in the background with VNC on :0
# We use -vga virtio for better browser compatibility
qemu-system-x86_64 \
    -m 512 \
    -drive format=raw,file=/app/jarvis-os.img \
    -vnc :0 \
    -nographic \
    -serial mon:stdio \
    &

# Start websockify to bridge VNC (5900) to WebSocket (8080) for noVNC
websockify --web=/usr/share/novnc/ 8080 localhost:5900
