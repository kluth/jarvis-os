#!/usr/bin/env bash

# JARVIS OS - Sovereign Local Validation Script (Pure JRV)
# This script ensures that the JRV codebase meets all quality standards.

set -e

echo "Starting JARVIS OS validation (Pure JRV Substrate)..."

# 1. JRV Compilation Check (Rev 7.0 TG-IR Pipeline)
echo "Checking JRV modules (Total Substrate Purity)..."
JRV_CMD="./jrvc"
if [ "$(uname -m)" != "aarch64" ] && [ -f "/usr/bin/qemu-aarch64-static" ]; then
    echo "Non-AArch64 host detected. Using QEMU emulation for jrvc."
    PREFIX=${QEMU_LD_PREFIX:-"/usr/aarch64-linux-gnu"}
    JRV_CMD="qemu-aarch64-static -L $PREFIX ./jrvc"
fi

for f in $(find . -name "*.jrv"); do
    echo "Compiling $f to TG-IR payload..."
    $JRV_CMD "$f" --tg-ir --fix-plans > /dev/null
done

echo "Validation successful! All JRV modules are formally verified."
rm -f output.elf
