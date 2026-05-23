; ==========================================================
; JARVIS OS - Aetheris Spatial Interface v2 (Stitch-aligned)
; ----------------------------------------------------------
; Stage-1: real-mode boot sector. Loads Stage-2, sets VBE
;          mode 0x115 (800x600x24), enters protected mode.
; Stage-2: 32-bit per-pixel software renderer. Composites
;          the Aetheris spatial interface directly into the
;          linear framebuffer (BGR24 packed) in 10 layers.
; ----------------------------------------------------------
; Palette (Stitch tokens):
;   obsidian       = 0x0e0e0e   surface-container-lowest
;   glass          = 0x131a1d   panel fill
;   glass_header   = 0x161e22   panel header band
;   grid_line      = 0x121d20   spatial grid
;   border_cyan    = 0x003a45   cyan/20 border
;   neon_cyan      = 0x00daf3   accent / borders / voice (surface-tint)
;   bright_cyan    = 0xc3f5ff   scanline / outer ring (primary)
;   inner_orb      = 0x9cf0ff   pulsing core (primary-fixed)
;   active_tile    = 0x153a44   side-rail active item
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

    ; Load Stage-2 (32 sectors = 16 KiB, room for growth)
    mov ah, 0x02
    mov al, 32
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov bx, 0x7e00
    int 0x13

    ; Query VBE mode info for 0x115 -> writes ModeInfoBlock at 0x9000
    mov ax, 0x4f01
    mov cx, 0x115
    mov di, 0x9000
    int 0x10

    ; Set VBE mode 0x115 with LFB bit
    mov ax, 0x4f02
    mov bx, 0x4115
    int 0x10

    ; A20 + protected-mode entry
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
; STAGE 2 - 32-bit protected mode renderer
; ==========================================================
[bits 32]
start32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000        ; safe high stack

    xor ebp, ebp            ; frame counter

render_loop:
    inc ebp
    mov edi, [0x9028]       ; LFB physical address
    xor esi, esi            ; y = 0

.y_loop:
    xor ebx, ebx            ; x = 0

.x_loop:
    ; ----- Layer 0: deep obsidian background -----
    mov eax, 0x000e0e0e

    ; ----- Layer 1: 64x64 spatial grid -----
    test ebx, 63
    jz .grid_hit
    test esi, 63
    jnz .L2
.grid_hit:
    mov eax, 0x00121d20

.L2:
    ; ----- Layer 2: Top header bar (y < 48) -----
    cmp esi, 48
    jge .L3
    cmp esi, 46
    jge .hdr_border
    mov eax, 0x00131313     ; surface
    jmp .L3
.hdr_border:
    mov eax, 0x003b494c     ; outline-variant

.L3:
    ; ----- Layer 3: Left side rail (0..48, 192..448) -----
    cmp ebx, 0
    jl .L4
    cmp ebx, 48
    jge .L4
    cmp esi, 192
    jl .L4
    cmp esi, 448
    jge .L4
    cmp ebx, 1
    jle .rail_b
    cmp ebx, 46
    jge .rail_b
    cmp esi, 193
    jle .rail_b
    cmp esi, 447
    jge .rail_b
    mov eax, 0x000e0e0e     ; surface-container-lowest
    ; Active tile at y in [220..280]
    cmp esi, 220
    jl .L4
    cmp esi, 280
    jge .L4
    cmp ebx, 10
    jl .L4
    cmp ebx, 38
    jge .L4
    mov eax, 0x00153a44
    jmp .L4
.rail_b:
    mov eax, 0x003b494c

.L4:
    ; ----- Layer 4: Substrate Health panel (64..320, 64..320) -----
    cmp ebx, 64
    jl .L5
    cmp ebx, 320
    jge .L5
    cmp esi, 64
    jl .L5
    cmp esi, 320
    jge .L5
    cmp ebx, 65
    jle .p1_b
    cmp ebx, 318
    jge .p1_b
    cmp esi, 65
    jle .p1_b
    cmp esi, 318
    jge .p1_b
    mov eax, 0x001c1b1b     ; surface-container-low
    ; Header band: y in [65..96]
    cmp esi, 96
    jge .p1_check_status
    mov eax, 0x00201f1f     ; surface-container
    jmp .L5
.p1_check_status:
    ; "WASM_SANDBOX: SECURE" green status dot at (80, 280) r=4
    mov ecx, ebx
    sub ecx, 80
    imul ecx, ecx
    mov edx, esi
    sub edx, 280
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .L5
    mov eax, 0x0034d399     ; emerald-400
    jmp .L5
.p1_b:
    mov eax, 0x003b494c

.L5:
    ; ----- Layer 5: Swarm Visualizer panel (448..768, 64..448) -----
    cmp ebx, 448
    jl .L6
    cmp ebx, 768
    jge .L6
    cmp esi, 64
    jl .L6
    cmp esi, 448
    jge .L6
    cmp ebx, 449
    jle .p2_b
    cmp ebx, 767
    jge .p2_b
    cmp esi, 65
    jle .p2_b
    cmp esi, 447
    jge .p2_b
    mov eax, 0x001c1b1b
    cmp esi, 96
    jge .p2_marker
    mov eax, 0x00201f1f
    jmp .L6
.p2_marker:
    ; Interactive node marker 1: (576, 200) r=4 cyan ring
    mov ecx, ebx
    sub ecx, 576
    imul ecx, ecx
    mov edx, esi
    sub edx, 200
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .p2_marker2
    cmp ecx, 4
    jl .L6
    mov eax, 0x0000daf3
    jmp .L6
.p2_marker2:
    ; Interactive node marker 2: (680, 360) r=4 lavender ring
    mov ecx, ebx
    sub ecx, 680
    imul ecx, ecx
    mov edx, esi
    sub edx, 360
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .L6
    cmp ecx, 4
    jl .L6
    mov eax, 0x00b6c4ff     ; secondary-fixed-dim
    jmp .L6
.p2_b:
    mov eax, 0x003b494c

.L6:
    ; ----- Layer 6: Evolution Log panel (200..600, 480..560) -----
    cmp ebx, 200
    jl .L7
    cmp ebx, 600
    jge .L7
    cmp esi, 480
    jl .L7
    cmp esi, 560
    jge .L7
    cmp ebx, 201
    jle .p3_b
    cmp ebx, 599
    jge .p3_b
    cmp esi, 481
    jle .p3_b
    cmp esi, 559
    jge .p3_b
    mov eax, 0x001c1b1b
    ; Header strip y in [481..500]
    cmp esi, 500
    jge .p3_lines
    mov eax, 0x00201f1f
    jmp .L7
.p3_lines:
    mov ecx, esi
    sub ecx, 510
    cmp ecx, 0
    jl .L7
    test ecx, 7
    jnz .L7
    cmp ebx, 210
    jl .L7
    cmp ebx, 230
    jge .L7
    mov eax, 0x0000daf3
    jmp .L7
.p3_b:
    mov eax, 0x003b494c

.L7:
    ; ----- Layer 7: Thought Core (center 400, 256) -----
    mov ecx, ebx
    sub ecx, 400
    imul ecx, ecx
    mov edx, esi
    sub edx, 256
    imul edx, edx
    add ecx, edx            ; ecx = dist^2 from core center

    cmp ecx, 36864          ; r > 192 -> outside halo
    jg .L8
    cmp ecx, 16384          ; r > 128 -> halo band
    jg .core_halo
    cmp ecx, 14400          ; r in [120..128] -> sphere border
    jge .core_border
    cmp ecx, 11236          ; r > 106 -> sphere inner glass
    jg .core_glass
    cmp ecx, 9216           ; r in [96..106] -> dashed orbit
    jg .core_orbit
    ; Pulsing inner orb: r in [72..95], modulated by frame
    mov edx, ebp
    shr edx, 3
    and edx, 23
    add edx, 72
    imul edx, edx           ; pulse r^2
    cmp ecx, edx
    jg .core_outer_orb
    mov eax, 0x009cf0ff     ; hot inner orb (primary-fixed)
    jmp .L8
.core_outer_orb:
    mov eax, 0x00003a45     ; cool outer orb (border-cyan)
    jmp .L8
.core_orbit:
    ; Dashed orbital ring -- sample by angle approximation via (x XOR y) & 8
    mov edx, ebx
    xor edx, esi
    test edx, 8
    jz .L8                  ; gap
    mov eax, 0x0000daf3
    jmp .L8
.core_glass:
    mov eax, 0x001c1b1b     ; glass substrate
    jmp .L8
.core_border:
    mov eax, 0x0000daf3
    jmp .L8
.core_halo:
    ; Additive halo tint over whatever Layer<7 painted
    add eax, 0x00020405

.L8:
    ; ----- Layer 8: Animated scanline across core -----
    ; scan_y = 128 + ((ebp>>1) & 255)  -> sweeps 128..383
    mov ecx, ebp
    shr ecx, 1
    and ecx, 255
    add ecx, 128
    mov edx, esi
    sub edx, ecx
    cmp edx, -1
    jl .L9
    cmp edx, 1
    jg .L9
    ; Constrain to sphere x range (|x-400| < 120)
    mov edx, ebx
    sub edx, 400
    cmp edx, -120
    jl .L9
    cmp edx, 120
    jg .L9
    mov eax, 0x00c3f5ff

.L9:
    ; ----- Layer 9a: Bottom nav pill (280..520, 520..560) -----
    cmp ebx, 280
    jl .L9_voice
    cmp ebx, 520
    jge .L9_voice
    cmp esi, 520
    jl .L9_voice
    cmp esi, 560
    jge .L9_voice
    mov eax, 0x00131313
    ; Top border
    cmp esi, 521
    jle .pill_b
    jmp .L9_voice
.pill_b:
    mov eax, 0x003b494c

.L9_voice:
    ; ----- Layer 9b: Voice button (cx=400, cy=512, r=32) -----
    mov ecx, ebx
    sub ecx, 400
    imul ecx, ecx
    mov edx, esi
    sub edx, 512
    imul edx, edx
    add ecx, edx
    cmp ecx, 1024           ; r > 32 -> outside
    jg .L10
    cmp ecx, 784            ; r in [28..32] -> ring
    jge .voice_ring
    mov eax, 0x0000daf3     ; solid cyan core
    jmp .L10
.voice_ring:
    mov eax, 0x00c3f5ff

.L10:
    ; ----- Layer 10: Ambient agent nodes -----
    ; Node A: (200, 150) r=3, pulsing dim cyan
    mov ecx, ebx
    sub ecx, 200
    imul ecx, ecx
    mov edx, esi
    sub edx, 150
    imul edx, edx
    add ecx, edx
    cmp ecx, 9
    jg .agentB
    mov eax, 0x0000daf3
    jmp .draw
.agentB:
    ; Node B: (600, 400) r=4, deep blue
    mov ecx, ebx
    sub ecx, 600
    imul ecx, ecx
    mov edx, esi
    sub edx, 400
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .draw
    mov eax, 0x000356ff     ; secondary-container

.draw:
    ; BGR24 packed: write B,G,R (3 bytes per pixel)
    stosw                   ; B (al) then G (ah) -> FB[0], FB[1]
    shr eax, 16
    stosb                   ; R (low byte) -> FB[2]
    inc ebx
    cmp ebx, 800
    jl .x_loop

    inc esi
    cmp esi, 600
    jl .y_loop

    ; VSync wait
    mov edx, 0x3DA
.wait_retrace:
    in al, dx
    test al, 8
    jz .wait_retrace

    jmp render_loop

; Pad to 33 sectors total (1 stage-1 + 32 stage-2 = 16896 bytes)
times 16896-($-$$) db 0
