FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

# Install QEMU, noVNC, websockify, socat, and netcat
RUN apt-get update && apt-get install -y \
    qemu-system-x86 \
    novnc \
    websockify \
    socat \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

# Create working directory
WORKDIR /app

# Create a non-privileged user
RUN useradd -m jarvis && chown -R jarvis:jarvis /app
USER jarvis

# Copy the latest kernel image
COPY --chown=jarvis:jarvis jarvis-os.img /app/jarvis-os.img

# Port for noVNC
EXPOSE 8080

# Start script
COPY --chown=jarvis:jarvis docker/start.sh /app/start.sh
RUN chmod +x /app/start.sh

ENTRYPOINT ["/app/start.sh"]
