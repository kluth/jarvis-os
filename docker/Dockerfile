FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

# Install QEMU, noVNC, and websockify
RUN apt-get update && apt-get install -y \
    qemu-system-x86 \
    novnc \
    websockify \
    && rm -rf /var/lib/apt/lists/*

# Create working directory
WORKDIR /app

# Copy the latest kernel image (to be mounted or embedded)
# We expect the image at /app/jarvis-os.img

# Port for noVNC
EXPOSE 8080

# Start script
COPY docker/start.sh /app/start.sh
RUN chmod +x /app/start.sh

ENTRYPOINT ["/app/start.sh"]
