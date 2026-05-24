#!/usr/bin/env bash

# JARVIS OS - Sovereign Build Script (Pure JRV)
# This script assembles the operating system from pure .jrv sources.

set -e

echo "Building JARVIS OS (100% JRV Substrate)..."

# 1. Compile the Core OS into a Sovereign Payload
# NOTE: Currently outputs as output.elf. 
# Pending issue #28 for bootable image emission.
./jrvc kernel.jrv --tg-ir --fix-plans

# 2. Package for Distribution
# (Simulation: Moving the ELF to the target image location)
mv output.elf jarvis-os.img

echo "Success: JARVIS OS Sovereign Payload generated (Pure JRV)."
