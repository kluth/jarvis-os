# ADR 0003: Memory Management Strategy

## Status
Proposed

## Context
The kernel requires efficient management of physical and virtual memory. Since x86_64 uses 4-level paging by default, we must decide how to make physical memory visible in the virtual address space and how to manage dynamic memory (heap).

## Decision
1.  **Physical Memory Mapping:** We use **Complete Physical Memory Mapping**. The entire physical memory is mapped linearly into a high range of the virtual address space (e.g., starting at offset `0x0000_4000_0000_0000`). This allows the kernel to access any physical address by simply adding the offset.
2.  **Frame Allocator:** For managing physical memory pages (frames), we initially implement a simple **Bump Allocator** based on the bootloader's memory map. This will later be replaced by a **Bitmap Allocator** or **Buddy Allocator** to support memory deallocation.
3.  **Page Table Management:** We utilize the `x86_64` crate abstractions (`OffsetPageTable`) to safely manipulate the 4-level page tables.
4.  **Heap Allocator:** We integrate the `linked_list_allocator` crate for the initial implementation of the kernel heap. This enables the use of `alloc` (Box, Vec, etc.) within the kernel.

## Bounded Context
**MemoryContext**: Responsible for allocating physical frames, mapping virtual pages, and managing the kernel heap.

## Design Patterns
*   **Resource Pool:** Management of physical frames.
*   **Singleton/Provider:** Global access to the allocator.

## Consequences
*   **Advantages:** Fast access to physical memory. Enables complex data structures through `alloc`.
*   **Disadvantages:** High virtual memory consumption for mapping (not problematic with x86_64). Initial bump allocator does not support frame deallocation.
