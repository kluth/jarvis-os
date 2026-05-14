# ADR 0002: Framebuffer Output and Provider Pattern

## Status
Proposed

## Context
The kernel needs a way to output information visually. Since we are using UEFI, the bootloader provides a framebuffer (linear buffer). We must decide how to structure access to this framebuffer to ensure thread safety (even before introducing scheduler/mutexes) and abstraction.

## Decision
1.  **Framebuffer Abstraction:** We implement a `Writer` that operates on the framebuffer provided by the bootloader.
2.  **Provider Pattern:** We use the Provider Pattern to provide a global instance of the writer. Since we don't have mutexes yet, we initially use a `Locked` abstraction (e.g., a simple spinlock or `lazy_static` with spinlock, if available, otherwise our own minimalist implementation).
3.  **Font:** For the first version, we use a simple, embedded bitmap font (e.g., 8x16) to draw text on the framebuffer.
4.  **Scrolling:** We implement a linear buffer scroll algorithm that shifts the memory content upwards when the screen is full.

## Bounded Context
**DisplayContext**: Responsible for controlling the graphics hardware (framebuffer) and rendering glyphs.

## Design Patterns
*   **Provider Pattern:** Global access to the display driver.
*   **Strategy Pattern (Preparation):** Abstraction of drawing operations to allow switching between different graphics modes or architectures later.

## Consequences
*   **Advantages:** Early visual feedback for debugging. Clear separation between text logic and pixel logic.
*   **Disadvantages:** Additional overhead for drawing pixels compared to VGA text mode (which is not guaranteed in UEFI).
