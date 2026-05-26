import sys
from PIL import Image
import numpy as np

def compare_images(design_path, os_path, output_diff_path):
    # Load images
    design = Image.open(design_path).convert('RGB')
    os_img = Image.open(os_path).convert('RGB')

    # Normalize resolution: Stitch is 2560x2048, JARVIS is 1280x1024
    # Downscale design to match OS
    design = design.resize(os_img.size, Image.Resampling.LANCZOS)

    # Convert to numpy arrays for fast comparison
    design_arr = np.array(design)
    os_arr = np.array(os_img)

    # Calculate differences
    diff = np.abs(design_arr.astype(np.int16) - os_arr.astype(np.int16))
    
    # Square error for MSE
    mse = np.mean(diff ** 2)
    
    # Create mask of mismatching pixels (threshold > 5 to ignore minor interpolation artifacts)
    mismatch_mask = np.any(diff > 5, axis=2)
    mismatch_count = np.sum(mismatch_mask)
    total_pixels = mismatch_mask.size
    mismatch_percent = (mismatch_count / total_pixels) * 100

    # Generate difference map: highlight mismatches in bright red
    diff_map = os_arr.copy()
    diff_map[mismatch_mask] = [255, 0, 0]
    
    # Save diff map
    Image.fromarray(diff_map).save(output_diff_path)

    print(f"--- PIXEL COMPARISON RESULTS ---")
    print(f"Resolution: {os_img.width}x{os_img.height}")
    print(f"Total Pixels: {total_pixels}")
    print(f"Mismatched Pixels: {mismatch_count}")
    print(f"Mismatch Percentage: {mismatch_percent:.4f}%")
    print(f"Mean Squared Error (MSE): {mse:.4f}")
    print(f"Difference map saved to: {output_diff_path}")

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python3 pixel_compare.py <design.png> <os.png> <diff_map.png>")
        sys.exit(1)
    
    compare_images(sys.argv[1], sys.argv[2], sys.argv[3])
