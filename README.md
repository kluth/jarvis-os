# J.A.R.V.I.S. OS (v0.2.0 - OMEGA)
**Just A Rather Very Intelligent System - Operating System**

JARVIS OS has undergone a complete OMEGA-LEVEL RECONSTRUCTION. The legacy Rust-based kernel has been replaced with a pure JRV implementation, leveraging the latest Project Jarvis substrate for zero-dependency, formal-verified, and AI-native execution.

---

## 🚀 Core Philosophy
- **Pure JRV:** The entire operating system logic is written in `.jrv` modules, utilizing eTDD (Evolutionary Test Driven Development) and complexity-bounded execution.
- **Aetheris Spatial Interface:** A high-fidelity, 32-bit protected mode software renderer built directly into the Stage-2 bootloader for instantaneous visual telemetry.
- **Voice-First Symbiosis:** Native integration with the JARVIS Multi-Agent Gateway (MAG) for autonomous environmental orchestration.

---

## 🛠 Architectural Highlights
- **Stitch-Aligned Design:** Visual tokens and layout models aligned with the Stitch design system.
- **Wait-Free Synchronization:** RCU (Read-Copy-Update) and atomic operations ensure real-time stability across distributed agent nodes.
- **Formal Verification:** In-kernel `verify` blocks and `budget` constraints enforced by the `jrvc` compiler.

---

## 🛠 Building & Running

### Prerequisites
- `nasm` (for the Aetheris bootloader)
- `jrvc` (the JRV AOT compiler - included)
- QEMU (for virtualization)

### Local Validation
Before pushing, always run the new JRV validation framework:
```bash
./scripts/validate.sh
```

### Build & Run
The system can be assembled using NASM and the JRV toolchain.
```bash
# Assemble the bootloader
nasm -f bin boot.asm -o boot.bin
# (Future: Compile and link JRV modules)
```

### 🐳 Docker Experience
```bash
cd docker && docker-compose up --build -d
```
Open **http://localhost:8080** in your browser to witness the Aetheris Spatial Interface.

---

## 📜 Development Standards
See [GEMINI.md](./GEMINI.md) for the latest JRV engineering standards.

---

© 2026 JARVIS Project. The Future is Pure.
