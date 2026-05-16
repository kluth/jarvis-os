#!/bin/bash

# JARVIS OS - One-Liner Run Script
# This script launches JARVIS OS in a Docker container with noVNC browser access.

set -e

IMAGE="ghcr.io/kluth/jarvis-os:latest"
CONTAINER_NAME="jarvis-os-live"

echo "--------------------------------------------------"
echo "   J.A.R.V.I.S. OS - Deployment Script"
echo "--------------------------------------------------"

# 1. Check for Docker
if ! [ -x "$(command -v docker)" ]; then
  echo "Error: Docker is not installed. Please install Docker first." >&2
  exit 1
fi

# 2. Check for KVM
KVM_DEVICE=""
if [ -e /dev/kvm ]; then
    echo "Found /dev/kvm - Enabling hardware acceleration."
    KVM_DEVICE="--device /dev/kvm"
else
    echo "Warning: /dev/kvm not found. System will run in emulation mode (slower)."
fi

# 3. Pull latest image
echo "Pulling latest JARVIS OS image..."
docker pull $IMAGE

# 4. Stop existing container if any
if [ "$(docker ps -aq -f name=$CONTAINER_NAME)" ]; then
    echo "Removing existing JARVIS OS container..."
    docker rm -f $CONTAINER_NAME > /dev/null
fi

# 5. Run JARVIS OS
echo "Starting JARVIS OS..."
docker run -d \
    --name $CONTAINER_NAME \
    $KVM_DEVICE \
    -p 8080:8080 \
    --restart unless-stopped \
    $IMAGE

echo ""
echo "--------------------------------------------------"
echo "🚀 JARVIS OS IS BOOTING!"
echo "--------------------------------------------------"
echo "Open your browser at: http://localhost:8080"
echo "--------------------------------------------------"
echo "To stop: docker stop $CONTAINER_NAME"
echo "To logs: docker logs -f $CONTAINER_NAME"
echo "--------------------------------------------------"
