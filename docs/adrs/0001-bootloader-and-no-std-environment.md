# ADR 0001: Bootloader and Rust no_std Environment

## Status
Accepted

## Context
For the development of JARVIS OS "from scratch", we need an environment that can run without an existing operating system (bare metal). The initial target architecture is x86_64, as it provides a broad hardware base and can be excellently emulated in QEMU. We must decide how the system starts (bootloader) and how to configure Rust in this environment.

## Decision
1.  **Programming Language:** We use **Rust** with the `#![no_std]` attribute to exclude the standard library (which assumes an OS). We utilize the `core` library.
2.  **Bootloader:** We use **UEFI (Unified Extensible Firmware Interface)** instead of the outdated BIOS. UEFI provides modern features such as memory mapping and graphics initialization (GOP) even before the kernel starts.
3.  **Bootloader Implementation:** We use the `bootloader` crate (v0.11 or higher) because it offers seamless integration into the Rust build process and puts the kernel directly into 64-bit Long Mode.
4.  **Architecture Patterns:**
    *   **Abstract Factory:** We define traits for basic hardware interactions (e.g., `Writer`, `Hal`) to facilitate porting the kernel to other architectures (ARM, RISC-V) later.
    *   **Chain of Responsibility:** The boot process is divided into clearly defined stages (UEFI -> Bootloader -> Kernel Init -> Module Init).

## Consequences
*   **Advantages:** 
    *   Full control over the hardware from the first instruction.
    *   Type safety and memory safety through Rust even in the kernel.
    *   Future-proof booting process through UEFI.
*   **Disadvantages:**
    *   High initial overhead for setup (cross-compilation, JSON targets).
    *   No use of standard Rust features like `std::vec` or `std::string` without a custom allocator.

## Verification
*   Compilation with `cargo build --target x86_64-unknown-none`.
*   Successful boot process in **QEMU**, indicated by framebuffer output.
