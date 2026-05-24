from PIL import Image
import numpy as np

img = Image.open('design_v10_scaled.png').convert('RGB')
arr = np.array(img)

print("--- Structural Sampling ---")
for y in range(0, 1024, 64):
    for x in range(0, 1280, 64):
        block = arr[y:y+64, x:x+64]
        avg = np.mean(block, axis=(0,1))
        # If block is not background obsidian (14,14,14)
        if np.max(np.abs(avg - [14, 14, 14])) > 5:
            print(f"Panel at {x},{y} - Color {avg}")

# Locate the core again in scaled space
bright_mask = np.mean(arr, axis=2) > 150
y_c, x_c = np.where(bright_mask)
if len(x_c) > 0:
    print(f"\nCore Center: ({np.mean(x_c):.0f}, {np.mean(y_c):.0f})")
    print(f"Core Radius: {(np.max(x_c) - np.min(x_c))/2:.0f}")
