#!/usr/bin/env bash

# JARVIS OS - Local Validation Script
# This script ensures that the JRV codebase meets all quality standards.

set -e

echo "Starting JARVIS OS validation (JRV Edition)..."

# 1. Bootloader Check
echo "Checking bootloader assembly..."
nasm -f bin boot.asm -o boot.bin
rm boot.bin

# 2. JRV Compilation Check
echo "Checking JRV modules..."
JRV_CMD="./jrvc"
if [ "$(uname -m)" != "aarch64" ] && [ -f "/usr/bin/qemu-aarch64-static" ]; then
    echo "Non-AArch64 host detected. Using QEMU emulation for jrvc."
    # Use QEMU_LD_PREFIX if set, otherwise default to standard cross-path
    PREFIX=${QEMU_LD_PREFIX:-"/usr/aarch64-linux-gnu"}
    JRV_CMD="qemu-aarch64-static -L $PREFIX ./jrvc"
fi

for f in $(find . -name "*.jrv"); do
    echo "Compiling $f..."
    $JRV_CMD "$f" > /dev/null
done

# 3. Documentation Check
if [ -f "GEMINI.md" ]; then
    echo "GEMINI.md found."
fi

echo "Validation successful! All JRV modules compiled and bootloader is valid."
rm -f output.elf
