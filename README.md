# JARVIS OS

The all-encompassing operating system project.

## CI/CD Status
Compilation is automated on GitHub to save local resources.

[![Build JARVIS OS](https://github.com/kluth/jarvis-os/actions/workflows/build.yml/badge.svg)](https://github.com/kluth/jarvis-os/actions/workflows/build.yml)

## Local Development
Since the build process is highly resource-intensive, it is recommended to download artifacts from GitHub Actions.

To test in QEMU (locally):
1. Download the `jarvis-kernel` artifact.
2. `qemu-system-x86_64 -drive format=raw,file=target/x86_64-jarvis_os/debug/jarvis-kernel` (adjust path).

*Note: Full disk image tooling will be finalized in Phase 1.*
