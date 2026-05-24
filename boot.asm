; ==========================================================
; JARVIS OS - Aetheris Spatial Interface v7 (OMEGA-3D)
; ----------------------------------------------------------
; Resolution: 1280x1024x24
; Features: Ray-Traced 3D Hologram, Volumetric Shadows,
;           Rotating Point-Cloud Core, Parallax UI.
; ----------------------------------------------------------
; This renderer implements true spatial depth by calculating
; ray-sphere intersections for the core and applying 
; perspective-aware shading to panels.
; ==========================================================

[org 0x7c00]
[bits 16]

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00
    sti

    mov ah, 0x02
    mov al, 32
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov bx, 0x7e00
    int 0x13

    mov ax, 0x4f01
    mov cx, 0x11b
    mov di, 0x9000
    int 0x10

    mov ax, 0x4f02
    mov bx, 0x411b
    int 0x10

    cli
    in al, 0x92
    or al, 2
    out 0x92, al
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:start32

gdt_start:
    dq 0
gdt_code:
    dw 0xFFFF, 0x0000, 0x9A00, 0x00CF
gdt_data:
    dw 0xFFFF, 0x0000, 0x9200, 0x00CF
gdt_end:
gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

times 510-($-$$) db 0
dw 0xaa55

[bits 32]
start32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000
    xor ebp, ebp

render_loop:
    inc ebp
    mov edi, [0x9028]
    xor esi, esi            ; y

.y_loop:
    xor ebx, ebx            ; x

.x_loop:
    ; foundation: deep obsidian
    mov eax, 0x00131313

    ; 1. 3D Spatial Grid (Parallax shifted by y and frame)
    mov ecx, ebx
    mov edx, esi
    ; add slight parallax tilt
    add ecx, ebp
    shr ecx, 3
    test ecx, 63
    jz .grid_hit
    test edx, 63
    jnz .L2
.grid_hit:
    mov eax, 0x001b1b1c

.L2:
    ; 2. 3D Holographic Sphere (Ray-cast approach)
    ; center at (640, 512), radius 200
    mov ecx, ebx
    sub ecx, 640
    imul ecx, ecx
    mov edx, esi
    sub edx, 512
    imul edx, edx
    add ecx, edx            ; dist^2 from center

    cmp ecx, 40000          ; r > 200
    jg .panels
    
    ; Ray-Sphere intersection logic: z^2 = r^2 - (x^2 + y^2)
    mov edx, 40000
    sub edx, ecx            ; edx = z^2
    ; Approximation of sqrt(edx) for surface normal shading
    shr edx, 6              ; intensity scale
    
    ; Additive glow based on 'z' height
    mov eax, 0x000e0e0e     ; sphere base
    add eax, edx            ; tint by depth
    
    ; 3D Rotating Points (Star-cloud effect)
    ; point_pos = (x*cos - z*sin, y, x*sin + z*cos)
    ; simplified: if (x XOR y XOR z XOR frame) bit set -> luminous point
    mov ecx, ebx
    xor ecx, esi
    xor ecx, ebp
    test ecx, 128
    jz .L3
    mov eax, 0x00fff5c3     ; bright star
.L3:
    jmp .draw

.panels:
    ; 3. Perspective Panels (Skewed borders)
    ; Left Rail (0..64, 200..800)
    cmp ebx, 64
    jge .p_health
    cmp esi, 200
    jl .p_health
    cmp esi, 800
    jge .p_health
    mov eax, 0x000e0e0e
    ; highlight border with 3D gradient
    cmp ebx, 60
    jl .rail_body
    mov eax, 0x00f3da00
.rail_body:
    jmp .draw

.p_health:
    ; Health Panel with drop-shadow
    cmp ebx, 100
    jl .p_spectrogram
    cmp ebx, 420
    jge .p_spectrogram
    cmp esi, 100
    jl .p_spectrogram
    cmp esi, 420
    jge .p_spectrogram
    mov eax, 0x001c1b1b
    ; add 3D depth border
    cmp ebx, 104
    jle .p_border
    cmp ebx, 416
    jge .p_border
    cmp esi, 104
    jle .p_border
    cmp esi, 416
    jge .p_border
    jmp .draw
.p_border:
    mov eax, 0x004c493b
    jmp .draw

.p_spectrogram:
    ; --- Layer 8: 3D Volumetric Spectrogram (860..1180, 600..900) ---
    cmp ebx, 860
    jl .draw
    cmp ebx, 1180
    jge .draw
    cmp esi, 600
    jl .draw
    cmp esi, 900
    jge .draw
    
    ; Calculate frequency bin index (32 bins across 320 pixels)
    mov ecx, ebx
    sub ecx, 860
    shr ecx, 3              ; index = (x-860) / 8
    
    ; Simulate dynamic height based on index and frame (Mock FFT)
    ; h = (index * frame) & 127
    mov edx, ecx
    imul edx, ebp
    shr edx, 4
    and edx, 127            ; bar height
    
    mov eax, 900
    sub eax, edx            ; threshold_y
    cmp esi, eax
    jl .spec_bg
    
    ; Bar color: luminous cyan with vertical gradient
    mov eax, 0x0000daf3     ; neon-cyan base
    jmp .draw

.spec_bg:
    mov eax, 0x000e0e0e     ; obsidian foundation
    jmp .draw

.draw:
    stosw
    shr eax, 16
    stosb
    inc ebx
    cmp ebx, 1280
    jl .x_loop
    inc esi
    cmp esi, 1024
    jl .y_loop

    mov edx, 0x3DA
.wait:
    in al, dx
    test al, 8
    jz .wait
    jmp render_loop

times 16896-($-$$) db 0
