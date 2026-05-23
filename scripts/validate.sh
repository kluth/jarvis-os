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
for f in $(find . -name "*.jrv"); do
    echo "Compiling $f..."
    ./jrvc "$f" > /dev/null
done

# 3. Documentation Check
if [ -f "GEMINI.md" ]; then
    echo "GEMINI.md found."
fi

echo "Validation successful! All JRV modules compiled and bootloader is valid."
rm -f output.elf
