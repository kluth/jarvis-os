# ADR 0005: Voice-Optimized Storage Strategy

## Status
Proposed

## Context
As a Voice-First Operating System, JARVIS OS handles continuous, high-bandwidth audio streams (recording for Speech-to-Text, playback for Text-to-Speech) and large continuous reads for AI models. The initial plan (Issue #4) proposed building a FAT32 file system. However, standard FAT file systems introduce significant metadata overhead (updating the File Allocation Table) which causes latency spikes—a critical flaw for real-time bare-metal audio processing. We must decide on a storage architecture that guarantees low latency, high throughput, and resilience.

## Decision
1. **Virtual File System (VFS) Layer:** We will implement a VFS abstraction layer. This ensures that the audio and AI subsystems do not interact directly with block devices.
2. **Custom Jarvis File System (JFS):** For internal storage, we will engineer a custom **Log-Structured, Append-Only File System**. 
   - *Why?* Audio logging and model caching are essentially sequential data streams. An append-only structure eliminates seek times for metadata updates, offering maximum throughput and natural crash consistency (if power is lost, we only lose the last unwritten block, and recovery is a simple linear scan).
3. **Outsourced FAT32 for Interoperability:** We will *not* develop a FAT32 driver from scratch. For reading external media (e.g., USB sticks with config files), we will wrap an existing, proven `no_std` crate (such as `fatfs` or `embedded-sdmmc`) behind our VFS interface.

## Bounded Context
**StorageContext**: Responsible for block device drivers (e.g., AHCI/NVMe in the future), the VFS routing, and the JFS implementation.

## Design Patterns
* **Facade Pattern (VFS):** Hides the complexity of different file systems behind a unified `open`, `read`, `write`, `stream` interface.
* **Log-Structured Storage:** Sequential, append-only writes with periodic garbage collection (compaction) in the background.

## Consequences
* **Advantages:** Unmatched performance for audio streaming; crash resilience; significantly less complex to implement securely than a fully-featured read/write FAT32 driver.
* **Disadvantages:** Internal files cannot be read natively by a Windows/Mac PC without a custom extractor tool; requires implementing a background compaction task to reclaim space from deleted/overwritten logs.
