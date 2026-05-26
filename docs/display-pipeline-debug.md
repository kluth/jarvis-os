# JARVIS OS — Display Pipeline: Deep Debugging Reference

## Status (2026-05-26)

**Kernel boots successfully** with all subsystems (ACPI, PCI, I2C, GPU, Net, Storage, Mesh, Onion, Security). Heartbeats tick. The GPU correctly detects **24bpp** from VBE registers and skips re-init. The framebuffer is remapped via phys-mem-offset. Test color bands are written and `wbinvd` flushed.

**Screendump is still ALL BLACK** despite all the above working. The QEMU VGA emulation does not see the pixels we write.

---

## What Has Been Fixed

### ✅ VBE bpp mismatch (24 vs 32)
- Root cause: bootloader initializes VBE to **1280×720×24bpp** (3 B/pixel, Bgr format), but `GpuManager::select_and_init()` hardcoded `bpp: 32`
- Fix: `select_and_init()` now queries VBE registers (`vbe_read_bpp/width/height`) and matches the bootloader's mode, so `BochsVbeDriver::init()` **skips** DISABLE/RE-ENABLE (the operation that wipes the display pipeline)
- Files: `src/gpu/mod.rs` (lines 1097–1120), `src/gpu/drivers.rs` (lines 95–109, 143–160)

### ✅ Framebuffer physical address from PCI BAR0
- Added `vbe_read_phys_addr()` to read the Bochs VGA framebuffer base address from PCI config space (BAR0, offset 0x10)
- Falls back from slot 1 to slot 2 for different QEMU configurations
- File: `src/gpu/drivers.rs` (lines 51–79)

### ✅ Phys-mem-offset framebuffer remap
- After `init_heap()` corrupts the bootloader's page tables, we re-map the framebuffer via the physical memory offset (a stable page table region set up by bootloader in reserved pages)
- Function: `vga_buffer::reinit_via_phys_mem_offset(phys_mem_offset, fb_phys_addr)`
- File: `src/vga_buffer.rs` (lines 240–256), used in `src/main.rs` (lines 67–83)

### ✅ Dynamic bytes-per-pixel computation
- `clear()`, `put_pixel()` in BochsVbeDriver now compute `bpp_div = (bpp + 7) / 8` instead of hardcoding `* 4`
- File: `src/gpu/drivers.rs` (lines 189–210)

### ✅ Framebuffer cache flush
- Added `flush_fb()` using `wbinvd` to flush CPU caches after framebuffer writes
- Called in `init_ui()`, `ui_task()` loop, and test section
- File: `src/vga_buffer.rs` (lines 258–263), `src/gui.rs` (lines 149, 252)

---

## Remaining Bug: "Screendump is all black"

Despite all the above fixes, QEMU `screendump` returns a PPM with 0 non-black pixels. The serial log confirms:
1. `VGA: remapped @ phys+0xfd000000=VA 0x200fd000000` — remap succeeds
2. `GPU: Selected Bochs/QEMU VBE @ 1280x720x24bpp` — bpp matches
3. `TEST: Wrote 4 color bands to framebuffer` — writes happen
4. `TEST: framebuffer flushed` — wbinvd called
5. No page faults

### Hypothesis: `ConcreteGpuDriver::detect_and_bind()` is Never Called

The GPU pipeline in `main.rs` looks like:
```rust
let mut gpu_manager = GpuManager::new();
gpu_manager.scan_pci();
let _ = gpu_manager.select_and_init(1280, 720);
```

But `GpuManager` only stores a `GpuMode` and PCI device metadata. **It never creates a `ConcreteGpuDriver` instance.** The `select_and_init()` method sets `self.current_mode` but does not call `ConcreteGpuDriver::detect_and_bind()`.

The result: **`BochsVbeDriver::init()` is never invoked** by `main.rs`. The VBE registers ARE being read (via the `vbe_read_*()` helpers we added), but the actual driver's `init()` method that sets up backbuffers, VRAM tracking, etc. is dead code.

### Hypothesis: `self.fb` is a Raw Physical Address

The Bochs driver's `present()` method copies the backbuffer to `self.fb` using `core::ptr::copy_nonoverlapping`:
```rust
fn present(&mut self) {
    unsafe {
        core::ptr::copy_nonoverlapping(
            self.backbuffer.as_ptr(), self.fb, self.backbuffer.len(),
        );
    }
}
```

Here `self.fb: *mut u8` is initialized from `ConcreteGpuDriver::detect_and_bind()` at line 1342:
```rust
let mut drv = BochsVbeDriver::new(fb);
```

Where `fb` comes from the function parameter `fb: *mut u8` at line 1327. Tracing the caller would reveal whether this is a virtual address (with phys-mem-offset added) or a raw physical address.

**If `self.fb` is a raw physical address like 0xFD000000**, then `copy_nonoverlapping` writes to physical address 0xFD000000 as if it were a virtual address, which would either page fault (if the address isn't mapped) or write to kernel memory (if something happens to be mapped there).

### Fix Strategy

**Step 1: Wire up the GPU driver init in main.rs**
Add a call to create the `ConcreteGpuDriver` and call `init()` + `present()`:
```rust
use jarvis_kernel::gpu::drivers::ConcreteGpuDriver;

// After select_and_init():
let fb_ptr = phys_mem_offset_raw as *mut u8; // placeholder — needs actual FB VA
let mut driver = ConcreteGpuDriver::detect_and_bind(
    &gpu_manager.devices[0],
    fb_ptr,
    1280, 720
);
let _ = driver.init(&gpu_manager.current_mode);
driver.present();  // flushes backbuffer to FB
```

**Step 2: Fix the `fb` pointer in `detect_and_bind`**
The `fb: *mut u8` parameter passed to `BochsVbeDriver::new()` must be a valid virtual address. Either:
- Add `phys_mem_offset + fb_phys_addr` to get a VA from the physical address
- Or pass the pre-mapped pointer from `reinit_via_phys_mem_offset`

**Step 3: Fix hardcoded bpp=32 in `detect_and_bind`**
Line 1327–1337 in `drivers.rs` hardcodes `bpp: 32` and `pitch: desired_w * 4`. This should use the VBE-detected bpp just like `select_and_init` now does.

### Debugging Checklist
- [ ] Can we verify `self.fb` value at runtime?
- [ ] Does `present()` cause a page fault? (It might not if the physical address happens to be in a mapped region via the identity map)
- [ ] What is the actual VA of phys_mem_offset + 0xFD000000? (0x200FD000000 based on serial log)
- [ ] Does a direct `write_nt()` to 0x200FD000000 work? (bypass FramebufferWriter, write directly)

## Boot Image Location
`/tmp/jarvis-test-24bpp.img` — latest build with all fixes, ready for QEMU testing.

## Git Branches
- `jarvis-os`: `pr-231-boot` (pushed to origin)
- `project-jarvis`: `chore/sync-compiler-updates` (pushed, needs PR into main)