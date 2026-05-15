# ADR 0008: Autonomous Discovery & Dynamic Capabilities

## Status
Proposed

## Context
JARVIS OS aims to be a proactive, voice-first artificial intelligence operating system. To achieve this, it must be aware of its physical and digital surroundings and capable of extending its functionality at runtime without requiring a full system reboot or manual driver installation. Currently, hardware discovery is limited to static PCI scanning, and there is no mechanism for dynamic network service discovery or sandboxed driver extension.

## Decision
We will implement an **Autonomous Discovery & Dynamic Adaptation** architecture based on four distinct layers:

1.  **Discovery Layer (Multi-Engine Scanning):**
    *   **Hardware:** Extend the PCI/USB scanners to prioritize **Class Codes** (e.g., UVC for cameras, HID for inputs) rather than specific Vendor/Device IDs.
    *   **Network (Zero-Config):** Implement background tasks for **mDNS/DNS-SD** (Port 5353), **SSDP/UPnP** (Port 1900), and **WS-Discovery/ONVIF** (Port 3702).
    *   **Wireless:** Future support for Bluetooth LE advertisement scanning.

2.  **Profiling Layer (Endpoint Analysis):**
    *   Implement an automated probing system that identifies device capabilities via standard protocols (HTTP/JSON, RTSP for video).
    *   Utilize local AI heuristics to analyze unknown JSON payloads to infer device types (e.g., matching keys like `temp_c` to temperature sensors).

3.  **Adaptation Layer (Sandboxed Wasm Plugins):**
    *   Adopt **WebAssembly (Wasm)** as the primary target for dynamic, third-party, and exotice device drivers.
    *   Wasm modules will run in a strictly isolated environment, communicating with the kernel through a standardized set of traits (e.g., `CameraTrait`, `SensorTrait`).
    *   Provide "Master Drivers" for industry standards (UVC, ONVIF) natively in the kernel.

4.  **Integration Layer (Dynamic Intent Mapping):**
    *   The Voice Shell will maintain a dynamic registry of system capabilities.
    *   Discovered devices will register their "Capabilities" (e.g., `VideoCapture`, `Location: "Living Room"`).
    *   The Intent Mapper will automatically link voice commands (e.g., "JARVIS, show me the living room") to the appropriate discovered hardware endpoints.

## Consequences
*   **Security:** Dynamic loading of code necessitates a robust sandbox (Wasm) to prevent malicious or buggy drivers from compromising kernel integrity.
*   **Complexity:** Network discovery protocols (especially SOAP/XML based ones like WS-Discovery) are heavy for a `no_std` environment and will require lean, custom implementations.
*   **Flexibility:** JARVIS OS becomes a dynamic orchestrator rather than a static kernel, allowing it to adapt to any environment (Smart Home, Industrial, etc.) autonomously.
*   **Resource Usage:** Constant discovery scanning will consume CPU cycles and network bandwidth; these tasks must be appropriately prioritized in the MLFQ scheduler.
