# Rev 7.0 Purity Manifest (Substrate Audit)

This document tracks the elimination of "Hollywood Code" (mocks, stubs, and simulated logic) across the JARVIS ecosystem, as mandated by the Rev 7.0 Substrate Purity initiative.

## 🔴 Critical Architectural Violations (Mocks)

### JARVIS OS (Rust/Hybrid)
- [ ] `src/net/mod.rs`: `discovery_task` uses hardcoded Node ID 2 (FRIDAY) and static keys.
- [ ] `src/net/ethernet.rs`: `init` manually registers a VirtIO NIC without actual driver logic.
- [ ] `src/net/mesh.rs`: `receive_next` returns hardcoded `None`.
- [ ] `mcp-bridge/index.ts`: `dispatch_instruction` simulates the handshake and queues instructions without real I/O.
- [ ] `src/ai/swarm.rs`: Health broadcast is simulated.
- [ ] `hal/ps2_input.jrv`: Uses `verifier-stubbed read`.

### Project JARVIS (Substrate)
- [ ] `compiler/inner_loop.jrv`: `verify` blocks use `tg_ir.mock_inner_loop_body`.
- [ ] `compiler/verifier.jrv`: Uses `tg_ir.mock_ast()`.
- [ ] `substrate/driver_synthesizer.jrv`: Relies on `tg_ir` for template selection and AST instantiation.
- [ ] `substrate/ui_compositor.jrv`: Uses `mock_scene_*`.

## 🟡 Partial Violations (Hardcoded Constants/Magics)
- [ ] `src/security.rs`: Fixed seed `0x42` and hardcoded "Top secret data" (Fixed in Phase 1.3 POC).
- [ ] `src/pci.rs`: Hardcoded class code `0x03` for display devices (should be dynamic).

## 🟢 Pure Substrate (Verified)
- [x] `src/entropy.rs`: Real hardware entropy (RDRAND) with MurmurHash3 fallback.
- [x] `src/acpi.rs`: Real RSDP/RSDT/XSDT parsing (verified in Phase 1.1).
- [x] `src/audio/hda.rs`: Real register-level HDA reset with timeouts (verified in Phase 1.1).

## Elimination Strategy
1. **Phase 1.3 (Current)**: Identify all violations and create formal `.jrv` module stubs (signatures only) for the target "Real" implementations.
2. **Phase 2-4**: Iteratively implement the formal modules, replacing `tg_ir` mocks with real calls.
3. **Phase 5**: Final verification of 100% purity.
