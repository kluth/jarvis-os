; ==========================================================
; JARVIS OS - Aetheris Spatial Interface v6 (3D VOLUMETRIC)
; ----------------------------------------------------------
; Resolution: 1280x1024x24 (5:4 Aspect, 0.5x Design Scale)
; Features: Volumetric Z-projection, real-time halo gradients,
;           spatial grid lattice, hardware-aligned logic.
; ----------------------------------------------------------
; Z-Axis Map (from designMd):
;   Background: Z = -100px
;   Main Plane: Z = 0
;   Foreground: Z = +50px
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

    ; Load Stage-2
    mov ah, 0x02
    mov al, 32
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov bx, 0x7e00
    int 0x13

    ; Query VBE mode info for 0x11b (1280x1024x24) -> ModeInfoBlock at 0x9000
    mov ax, 0x4f01
    mov cx, 0x11b
    mov di, 0x9000
    int 0x10

    ; Set VBE mode 0x11b with LFB bit
    mov ax, 0x4f02
    mov bx, 0x411b
    int 0x10

    ; Protected mode entry
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

; ==========================================================
; STAGE 2 - 32-bit VOLUMETRIC RENDERER
; ==========================================================
[bits 32]
start32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000
    xor ebp, ebp            ; Frame counter

render_loop:
    inc ebp
    mov edi, [0x9028]       ; LFB address
    xor esi, esi            ; y = 0

.y_loop:
    xor ebx, ebx            ; x = 0

.x_loop:
    ; --- Layer 0: Deep Obsidian (#131313) ---
    mov eax, 0x00131313

    ; --- Layer 1: Spatial Grid (64px interval) ---
    test ebx, 63
    jz .grid_hit
    test esi, 63
    jnz .L2
.grid_hit:
    mov eax, 0x001b1b1c     ; surface-container-low

.L2:
    ; --- Layer 2: Top Header (Z=0, h=64) ---
    cmp esi, 64
    jge .L3
    cmp esi, 62
    jge .hdr_border
    mov eax, 0x000e0e0e
    ; Title: "J.A.R.V.I.S" center 640
    cmp esi, 20
    jl .L3
    cmp esi, 30
    jge .L3
    cmp ebx, 600
    jl .L3
    cmp ebx, 680
    jge .L3
    mov eax, 0x00fff5c3
    jmp .L3
.hdr_border:
    mov eax, 0x004c493b

.L3:
    ; --- Layer 3: Left Side Rail (Z=20, 0..64, 256..768) ---
    cmp ebx, 64
    jge .L4
    cmp esi, 256
    jl .L4
    cmp esi, 768
    jge .L4
    ; Volumetric check (rounded r=16)
    cmp ebx, 62
    jge .rail_border
    mov eax, 0x000e0e0e
    ; Active Tile y:[320..384]
    cmp esi, 320
    jl .L4
    cmp esi, 384
    jge .L4
    cmp ebx, 12
    jl .L4
    cmp ebx, 52
    jge .L4
    mov eax, 0x00443a15     ; active-tile (BGR)
    jmp .L4
.rail_border:
    mov eax, 0x00f3da00     ; tint accent

.L4:
    ; --- Layer 4: Health Panel (Z=0, 128..448, 128..448) ---
    cmp ebx, 128
    jl .L5
    cmp ebx, 448
    jge .L5
    cmp esi, 128
    jl .L5
    cmp esi, 448
    jge .L5
    cmp ebx, 129
    jle .p1_b
    cmp ebx, 447
    jge .p1_b
    cmp esi, 129
    jle .p1_b
    cmp esi, 447
    jge .p1_b
    mov eax, 0x001c1b1b
    cmp esi, 160
    jge .p1_check_status
    mov eax, 0x001f1f20
    jmp .L5
.p1_check_status:
    ; "WASM_SANDBOX: SECURE" green status dot at (144, 400)
    mov ecx, ebx
    sub ecx, 144
    imul ecx, ecx
    mov edx, esi
    sub edx, 400
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .p1_gpu_status
    mov eax, 0x0034d399     ; emerald-400
    jmp .L5
.p1_gpu_status:
    ; "GPU_AMD: ACTIVE" neon-cyan status dot at (144, 424)
    mov ecx, ebx
    sub ecx, 144
    imul ecx, ecx
    mov edx, esi
    sub edx, 424
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .L5
    mov eax, 0x0000daf3     ; neon-cyan
    jmp .L5
.p1_b:
    mov eax, 0x004c493b

.L5:
    ; --- Layer 5: Swarm Panel (Z=-100, 768..1152, 128..512) ---
    cmp ebx, 768
    jl .L6
    cmp ebx, 1152
    jge .L6
    cmp esi, 128
    jl .L6
    cmp esi, 512
    jge .L6
    cmp ebx, 769
    jle .p2_b
    cmp ebx, 1151
    jge .p2_b
    cmp esi, 129
    jle .p2_b
    cmp esi, 511
    jge .p2_b
    mov eax, 0x001c1b1b
    cmp esi, 160
    jge .L6
    mov eax, 0x001f1f20
    jmp .L6
.p2_b:
    mov eax, 0x004c493b

.L6:
    ; --- Layer 6: Thought Core (Z=50, center 640, 512) ---
    mov ecx, ebx
    sub ecx, 640
    imul ecx, ecx
    mov edx, esi
    sub edx, 512
    imul edx, edx
    add ecx, edx            ; ecx = dist^2

    cmp ecx, 102400         ; r > 320 -> halo edge
    jg .L7
    
    ; Additive Halo (Linear falloff approx)
    mov edx, ecx
    shr edx, 10             ; scale dist
    neg edx
    add edx, 100            ; intensity
    cmp edx, 0
    jl .core_body
    shl edx, 1              ; tint
    add eax, edx

.core_body:
    cmp ecx, 25600          ; r > 160 -> sphere edge
    jg .L7
    cmp ecx, 23104          ; r in [152..160] -> border
    jge .core_border
    mov eax, 0x001c1b1b
    
    ; Pulsing Inner Orb (Z-modulated)
    mov edx, ebp
    shr edx, 2
    and edx, 31
    add edx, 96
    imul edx, edx
    cmp ecx, edx
    jg .L7
    mov eax, 0x00fff09c
    jmp .L7
.core_border:
    mov eax, 0x00f3da00

.L7:
    ; --- Layer 7: Scanline ---
    mov ecx, ebp
    shl ecx, 1
    and ecx, 511
    add ecx, 256            ; sweeps 256..767
    mov edx, esi
    sub edx, ecx
    cmp edx, -1
    jl .L8
    cmp edx, 1
    jg .L8
    ; Sphere mask check (|x-640| < 160)
    mov edx, ebx
    sub edx, 640
    cmp edx, -160
    jl .L8
    cmp edx, 160
    jg .L8
    mov eax, 0x00fff5c3

.L8:
    ; Write to VRAM (BGR24)
    stosw
    shr eax, 16
    stosb
    
    inc ebx
    cmp ebx, 1280
    jl .x_loop

    inc esi
    cmp esi, 1024
    jl .y_loop

    ; VSync
    mov edx, 0x3DA
.wait:
    in al, dx
    test al, 8
    jz .wait
    jmp render_loop

times 16896-($-$$) db 0
