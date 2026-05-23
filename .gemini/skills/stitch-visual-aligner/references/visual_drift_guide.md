# Visual Drift Guide

This guide defines the criteria for "perfect alignment" between JARVIS OS and Stitch.

## Alignment Metrics

### 1. Spatial Grid (64x64)
- **Constraint**: Every panel must align to the 64-pixel cyan lattice.
- **Drift**: If a panel border is at `x=110`, it must be shifted to `x=112` (the nearest grid ridge).

### 2. Stitch Token Colors
| Token | Hex | Usage |
| :--- | :--- | :--- |
| obsidian | #0e0e0e | Background |
| glass | #131a1d | Panel Fill |
| neon_cyan | #00daf3 | Active Borders / Highlights |
| bright_cyan | #b0f5ff | High-intensity Orbs / Scanlines |

### 3. Typography (Simulated)
- Since the bootloader uses pixel-font maps, ensuring "typography" means:
- **Baseline Alignment**: Text must be 16 pixels from the top of its glass container.
- **Kerning**: Fixed-width spacing must be 8 pixels for standard headers.

### 4. Interactive State
- Active items in the Side Rail must use the `#153a44` active-tile token.
- Hover states (if implemented) must utilize a 20% opacity cyan overlay.

## Patching Patterns

### Bootloader (`boot.asm`)
Look for the `Layer` blocks. Mismatch in coordinates usually happens here:
```nasm
    ; Incorrect
    cmp ebx, 100
    jl .skip
    cmp ebx, 200
    
    ; Correct (Grid-aligned)
    cmp ebx, 96
    jl .skip
    cmp ebx, 192
```

### JRV UI (`holographic_core.jrv`)
Update the `budget` and `contract` definitions to reflect new complexity requirements if the renderer becomes more sophisticated.
