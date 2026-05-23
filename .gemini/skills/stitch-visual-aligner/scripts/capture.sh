#!/bin/bash

# capture.sh — Capture a screenshot from the running JARVIS OS container.

set -e

CONTAINER_NAME="jarvis_vm" # Default from docker-compose.yml
OUTPUT_FILE=${1:-"screenshot.png"}
TEMP_PPM="/tmp/screenshot.ppm"

echo "Capturing screenshot from container $CONTAINER_NAME..."

# 1. Send screendump command to QEMU monitor inside the container
docker exec $CONTAINER_NAME bash -c "echo 'screendump $TEMP_PPM' | socat - /tmp/qemu-monitor.sock"

# 2. Copy the PPM file out of the container
docker cp $CONTAINER_NAME:$TEMP_PPM /tmp/jarvis_screenshot.ppm

# 3. Convert PPM to PNG using ffmpeg
ffmpeg -i /tmp/jarvis_screenshot.ppm -vframes 1 -q:v 2 "$OUTPUT_FILE" -y > /dev/null 2>&1

echo "Screenshot saved to $OUTPUT_FILE"

# Cleanup
rm /tmp/jarvis_screenshot.ppm
docker exec $CONTAINER_NAME rm $TEMP_PPM
