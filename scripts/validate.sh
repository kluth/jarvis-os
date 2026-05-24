#!/usr/bin/env bash

# JARVIS OS - Local Validation Script
# This script ensures that the JRV codebase meets all quality standards.

set -e

echo "Starting JARVIS OS validation (JRV Edition)..."

# 1. Bootloader Check
echo "Checking bootloader assembly..."
nasm -f bin boot.asm -o boot.bin
rm boot.bin

# 2. JRV Compilation Check (Rev 7.0 TG-IR Pipeline)
echo "Checking JRV modules (Sovereign Substrate Purity)..."
JRV_CMD="./jrvc"
if [ "$(uname -m)" != "aarch64" ] && [ -f "/usr/bin/qemu-aarch64-static" ]; then
    echo "Non-AArch64 host detected. Using QEMU emulation for jrvc."
    PREFIX=${QEMU_LD_PREFIX:-"/usr/aarch64-linux-gnu"}
    JRV_CMD="qemu-aarch64-static -L $PREFIX ./jrvc"
fi

for f in $(find . -name "*.jrv"); do
    echo "Compiling $f to TG-IR payload..."
    # --tg-ir: Output Target-Generic IR
    # --fix-plans: Generate machine-repair instructions on failure
    $JRV_CMD "$f" --tg-ir --fix-plans > /dev/null
done

# 3. Documentation Check
if [ -f "GEMINI.md" ]; then
    echo "GEMINI.md found."
fi

echo "Validation successful! All JRV modules compiled and bootloader is valid."
rm -f output.elf
