# ADR 0009: AI Autonomy - Possibilities & Limitations

## Status
Proposed

## Context
JARVIS OS is designed to be an autonomous, AI-driven operating system. This autonomy extends beyond simple discovery (ADR 0008) to high-level decision-making, dynamic capability synthesis, and environmental adaptation. However, the use of Large Language Models (LLMs) and other AI agents within a kernel context presents unique challenges and strict limitations.

## Research Findings

### 1. Possibilities of AI Autonomy
*   **Dynamic Driver Synthesis:** Using LLMs to interpret raw documentation (PDFs, GitHub issues) and generate sandboxed WebAssembly drivers at runtime.
*   **Intent-Based Resource Orchestration:** Moving away from static process scheduling to dynamic routing based on the semantic priority of user intent (e.g., prioritized audio pipeline when speech is detected).
*   **Self-Healing Hardware Paths:** Autonomous diagnosis of hardware failures with the ability to reroute I/O to alternative endpoints (e.g., using a discovered IP camera if the local UVC webcam fails).
*   **Zero-UI Discovery:** Proactive environmental awareness where the OS identifies services (mDNS/ONVIF) and prepares integrations before the user even asks.

### 2. Current Limitations
*   **Hallucinations in Kernel Space:** AI models can hallucinate memory addresses or hardware port values. In a `no_std` kernel, this can lead to immediate crashes or physical hardware damage. All AI decisions must be validated by a rule-based safety monitor.
*   **Latency in `no_std` environments:** Local inference on edge devices (especially x86/ARM embedded) is significantly slower than host-based execution. Synchronous I/O operations are currently not feasible for AI-based decision loops.
*   **Context Window Constraints:** Complex hardware specifications (e.g., Intel HDA) can exceed the context window or token limits for effective driver synthesis without extensive pre-processing.
*   **Non-Standardized Endpoints:** The fragmentation of IoT/Smart Home protocols forces the AI to "guess" capabilities via heuristic fingerprinting, which is prone to error and security vulnerabilities (e.g., spoofing).

## Decision
We will adopt a **"Sandboxed Agentic"** approach to autonomy:
1.  **AI is never the direct Actor:** AI agents synthesize plans and code (drivers/adapters), but a deterministic **"Safety Gate"** (deterministic Rust code) must validate and execute the low-level I/O.
2.  **WebAssembly is Mandatory:** All autonomously generated extensions MUST be sandboxed in WebAssembly to prevent kernel memory corruption.
3.  **Heuristic Confidence Thresholds:** Discovered endpoints must meet a confidence threshold (via fingerprinting) before being exposed to the Voice Shell.

## Consequences
*   **Architecture Complexity:** The kernel must now support a Wasm runtime and an asynchronous task-based agent system.
*   **Security Overhead:** The Safety Gate becomes a critical bottleneck and a primary target for security audits.
*   **Developer Experience:** Future developers must write "AI-readable" documentation for new modules to facilitate autonomous research (Phase 5 of AKA).
