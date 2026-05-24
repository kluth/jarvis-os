from PIL import Image
import numpy as np

img = Image.open('design_v10_scaled.png').convert('RGB')
arr = np.array(img)

def color_to_asm(c):
    # BGR for assembly
    return f"0x00{int(c[2]):02x}{int(c[1]):02x}{int(c[0]):02x}"

with open("boot_v11.asm", "w") as f:
    f.write("; AUTO-GENERATED PERFECTION RENDERER\n")
    f.write("[bits 32]\n")
    f.write("pixel_logic:\n")
    
    # Check for the core first (high priority)
    f.write("    ; --- Thought Core ---\n")
    f.write("    mov ecx, ebx\n")
    f.write("    sub ecx, 428\n")
    f.write("    imul ecx, ecx\n")
    f.write("    mov edx, esi\n")
    f.write("    sub edx, 749\n")
    f.write("    imul edx, edx\n")
    f.write("    add ecx, edx\n")
    f.write("    cmp ecx, 2500 ; r=50 pulse\n")
    f.write("    jg .panels\n")
    f.write(f"    mov eax, 0x00ffc6b8\n")
    f.write("    jmp .draw\n")
    
    f.write(".panels:\n")
    # Iterate through blocks and generate conditional logic
    for y in range(0, 1024, 64):
        f.write(f"    cmp esi, {y+64}\n")
        f.write(f"    jge .row_{y+64}\n")
        for x in range(0, 1280, 64):
            block = arr[y:y+64, x:x+64]
            avg = np.mean(block, axis=(0,1))
            if np.max(np.abs(avg - [14, 14, 14])) > 5:
                f.write(f"    cmp ebx, {x}\n")
                f.write(f"    jl .next_{x}_{y}\n")
                f.write(f"    cmp ebx, {x+64}\n")
                f.write(f"    jge .next_{x}_{y}\n")
                f.write(f"    mov eax, {color_to_asm(avg)}\n")
                f.write(f"    jmp .draw\n")
                f.write(f".next_{x}_{y}:\n")
        f.write(f"    jmp .row_{y+64}\n")
        f.write(f".row_{y+64}:\n")

    f.write(".draw:\n")
    f.write("    ret\n")
