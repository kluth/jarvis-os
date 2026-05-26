from PIL import Image
import numpy as np

# 1. Load and Resize Design to exactly 1280x1024
img = Image.open('stitch_design.png').convert('RGB')
img = img.resize((1280, 1024), Image.Resampling.LANCZOS)

# 2. Convert to BGR24 Raw Data
# PIL is RGB, we need BGR
r, g, b = img.split()
img_bgr = Image.merge('RGB', (b, g, r))
raw_data = np.array(img_bgr).tobytes()

# 3. Save as raw binary
with open('gui_data.bin', 'wb') as f:
    f.write(raw_data)

print(f"Generated gui_data.bin ({len(raw_data)} bytes)")
