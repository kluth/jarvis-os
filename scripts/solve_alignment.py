import numpy as np
from PIL import Image

def generate_gui_logic(width, height):
    arr = np.zeros((height, width, 3), dtype=np.uint8)
    
    # 0. Background
    arr[:,:] = [14, 14, 14] # #0e0e0e
    
    # 1. Grid
    for y in range(0, height, 64):
        arr[y, :] = [22, 29, 32]
    for x in range(0, width, 64):
        arr[:, x] = [22, 29, 32]
        
    # 2. Main Glass (Centered approx)
    # Scaled design check: (40, 102) to (1238, 958)
    gx, gy, gw, gh = 40, 102, 1198, 856
    arr[gy:gy+gh, gx:gx+gw] = [19, 26, 29] # #131a1d
    
    # 3. Thought Core (428, 749) r=100
    cx, cy, cr = 428, 749, 100
    for y in range(cy-cr, cy+cr):
        for x in range(cx-cr, cx+cr):
            if (x-cx)**2 + (y-cy)**2 <= cr**2:
                arr[y, x] = [184, 198, 255] # #b8c6ff
                
    return arr

def solve():
    design = Image.open('stitch_design.png').convert('RGB')
    design = design.resize((1280, 1024), Image.Resampling.LANCZOS)
    design_arr = np.array(design)
    
    # Generate current best guess
    solved_arr = generate_gui_logic(1280, 1024)
    
    # Compare
    diff = np.abs(design_arr.astype(np.int16) - solved_arr.astype(np.int16))
    mse = np.mean(diff ** 2)
    print(f"Solver MSE: {mse:.4f}")
    
    Image.fromarray(solved_arr).save("solved_guess.png")

if __name__ == "__main__":
    solve()
