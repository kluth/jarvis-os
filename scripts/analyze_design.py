import sys
from PIL import Image
import numpy as np

def analyze_design(path):
    img = Image.open(path).convert('RGB')
    arr = np.array(img)
    h, w, _ = arr.shape
    print(f"Design Dimensions: {w}x{h}")

    # 1. Sample Background (0,0)
    bg_color = tuple(arr[0,0])
    print(f"Background Color: {bg_color}")

    # 2. Find Panels (Look for #3b494c borders - [59, 73, 76] in RGB)
    # Actually, let's just find all unique colors and their usage
    unique_colors, counts = np.unique(arr.reshape(-1, 3), axis=0, return_counts=True)
    sorted_idx = np.argsort(-counts)
    print("\nTop colors used in design:")
    for i in range(min(10, len(unique_colors))):
        c = unique_colors[sorted_idx[i]]
        print(f"Color {c}: {counts[sorted_idx[i]]} pixels")

    # 3. Locate the core (Look for primary #c3f5ff - [195, 245, 255])
    primary_mask = np.all(np.abs(arr - [195, 245, 255]) < 50, axis=2)
    y_p, x_p = np.where(primary_mask)
    if len(x_p) > 0:
        print(f"\nPrimary color mass center: ({np.mean(x_p):.2f}, {np.mean(y_p):.2f})")
        print(f"Primary color bounds: x={np.min(x_p)}..{np.max(x_p)}, y={np.min(y_p)}..{np.max(y_p)}")

    # 4. Locate glass panels (Look for #131a1d - [19, 26, 29])
    glass_mask = np.all(np.abs(arr - [19, 26, 29]) < 5, axis=2)
    y_g, x_g = np.where(glass_mask)
    if len(x_g) > 0:
        print(f"\nGlass color mass center: ({np.mean(x_g):.2f}, {np.mean(y_g):.2f})")
        print(f"Glass color bounds: x={np.min(x_g)}..{np.max(x_g)}, y={np.min(y_g)}..{np.max(y_g)}")

if __name__ == "__main__":
    analyze_design("stitch_design.png")
