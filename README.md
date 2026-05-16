# J.A.R.V.I.S. OS (v0.1.0)
**Just A Rather Very Intelligent System - Operating System**

JARVIS OS is a highly modular, AI-native, voice-first operating system built in Rust. It is designed to autonomously discover, adapt to, and orchestrate its environment—from local hardware to network-distributed IoT services.

---

## 🚀 Core Philosophy
- **Voice-First:** Interaction is primarily speech-driven. The system communicates its state and capabilities through an audio pipeline.
- **AI-Native:** The kernel is orchestrated by an autonomous agent capable of synthesizing drivers and intent-based routing.
- **Extreme Modularity:** "Operation Small Footprint" ensures that only strictly necessary modules (GUI, Audio, AI, etc.) are compiled for the specific host hardware.
- **Autonomous Discovery:** JARVIS proactive identifies environmental endpoints using mDNS, SSDP, and ONVIF.

---

## 🛠 Architectural Highlights
- **Micro-Kernel Influence:** High isolation via sandboxed WebAssembly (Wasm) drivers.
- **Multi-Level Feedback Queue (MLFQ):** Priority-based task scheduling ensures real-time responsiveness for voice and safety critical tasks.
- **Zero-Warning Codebase:** Strictly enforced code quality (Rust, Clippy) with mandatory local validation.
- **Hardware Agnostic:** Flexible build system supporting various machine profiles (PC, Q35, etc.).

---

## 📸 System Previews (CI Generated)
*The following images are captured automatically during the CI/CD boot sequence.*

| Boot Stage 1 (Init) | Boot Stage 2 (Discovery) | Final State (Voice Shell) |
| :---: | :---: | :---: |
| ![Init](docs/images/screenshot_1.png) | ![Discovery](docs/images/screenshot_2.png) | ![Ready](docs/images/screenshot_3.png) |

*(Note: Real screenshots can be found in the [GitHub Actions Artifacts](https://github.com/kluth/jarvis-os/actions))*

---

## 🛠 Building & Running

### Prerequisites
- Rust Nightly (latest)
- QEMU
- `ffmpeg` (for visual artifacts)

### Local Validation (Recommended)
Before pushing, always run the validation framework:
```bash
./scripts/validate.sh
```

### Build & Run in QEMU
```bash
# Build the image-builder runner
cd image-builder && cargo run -- ../target/x86_64-jarvis_os/debug/jarvis-kernel
```

### 🐳 Quickstart with Docker
Experience JARVIS OS directly in your browser with a single command:
```bash
curl -sL https://raw.githubusercontent.com/kluth/jarvis-os/main/scripts/run.sh | bash
```
*Note: Requires Docker. KVM is recommended for performance.*

#### Manual Docker Setup (Optional)
1. Ensure you have the `jarvis-os.img` in `target/image/`.
2. Run the container:
   ```bash
   cd docker && docker-compose up --build -d
   ```
3. Open **http://localhost:8080** in your browser.

---

## 📜 Development Standards
See [GEMINI.md](./GEMINI.md) for detailed engineering standards, the "One Branch Per Feature" strategy, and safety protocols for Wakers and Interrupts.

---

## 🛡 Security & Safety
- **Safety Gate:** AI decisions are validated by deterministic Rust safety monitors.
- **Isolation:** Drivers run in a restricted Wasm sandbox to prevent kernel memory corruption.
- **Lock-Free:** Core OS components use atomic operations to avoid deadlocks in high-concurrency/ISR contexts.

---

© 2026 JARVIS Project. Built with 🦀 for the future.
