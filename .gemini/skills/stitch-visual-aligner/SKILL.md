---
name: stitch-visual-aligner
description: Automatically aligns the JARVIS OS interface with Stitch designs by capturing screenshots, identifying visual drift, and iteratively patching the codebase until pixel-perfect alignment is achieved.
---

# Stitch Visual Aligner

This skill automates the pixel-perfect alignment of the JARVIS OS Aetheris Spatial Interface with Stitch design specifications. It operates in an autonomous loop, capturing real-time telemetry from the running OS and applying surgical patches to the renderer.

## Workflow

1. **Capture**: Capture a real-time screenshot from the running QEMU/Docker instance.
2. **Retrieve Design**: Fetch the target screen design from the Stitch project.
3. **Analyze**: Compare the screenshot with the design to identify "Visual Drift" (discrepancies in colors, layout, spacing, or typography).
4. **Patch**: Apply surgical code changes to `boot.asm` (Stage-2 renderer) or `.jrv` UI modules.
5. **Verify**: Rebuild, relaunch, and re-capture to verify the fix.
6. **Iterate**: Repeat until the visual drift is zero.

## Key Resources

- **`scripts/capture.sh`**: Captures a `.png` screenshot from the `jarvis_vm` container.
- **`references/visual_drift_guide.md`**: Guidelines for identifying and fixing common alignment issues.

## Usage Instructions

### 1. Capturing a Screenshot
Run the capture script to get the current state of the OS:
```bash
./.gemini/skills/stitch-visual-aligner/scripts/capture.sh current_state.png
```

### 2. Identifying Drift
Use `mcp_stitch_get_screen` to get the target design. Compare it with `current_state.png`. Look for:
- **Color Inaccuracy**: Hex code mismatches vs. Stitch tokens.
- **Layout Shift**: Elements offset by pixels or incorrect grid alignment.
- **Dynamic Artifacts**: Z-index issues or rendering glitches in the 3D core.

### 3. Iterative Loop
The alignment loop MUST NOT stop until the OS perfectly reflects the Stitch design. If any drift remains, you are MANDATED to continue patching and verifying.

## Safety & Standards
- **Atomic Patches**: Only modify one visual component at a time (e.g., side-rail, then thought-core).
- **Validation**: Always run `./scripts/validate.sh` before relaunching the OS.
- **No Mocks**: Ensure the renderer uses real hardware values, not simulated pixels.
