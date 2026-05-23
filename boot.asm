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
;   neon_cyan      = 0x00daf3   accent / borders / voice
;   bright_cyan    = 0xb0f5ff   scanline / outer ring
;   inner_orb      = 0x80e8f5   pulsing core
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
    ; ----- Layer 2: Top header bar (y < 56) -----
    cmp esi, 56
    jge .L3
    cmp esi, 54
    jge .hdr_border
    mov eax, 0x00111719
    jmp .L3
.hdr_border:
    mov eax, 0x00003a45

.L3:
    ; ----- Layer 3: Left side rail (16..96, 200..420) -----
    cmp ebx, 16
    jl .L4
    cmp ebx, 96
    jge .L4
    cmp esi, 200
    jl .L4
    cmp esi, 420
    jge .L4
    cmp ebx, 18
    jle .rail_b
    cmp ebx, 94
    jge .rail_b
    cmp esi, 202
    jle .rail_b
    cmp esi, 418
    jge .rail_b
    mov eax, 0x00131a1d
    ; Active tile (top icon: HUB) at y in [220..280], x in [26..86]
    cmp esi, 220
    jl .L4
    cmp esi, 280
    jge .L4
    cmp ebx, 26
    jl .L4
    cmp ebx, 86
    jge .L4
    mov eax, 0x00153a44
    jmp .L4
.rail_b:
    mov eax, 0x00003a45

.L4:
    ; ----- Layer 4: Substrate Health panel (112..384, 80..280) -----
    cmp ebx, 112
    jl .L5
    cmp ebx, 384
    jge .L5
    cmp esi, 80
    jl .L5
    cmp esi, 280
    jge .L5
    cmp ebx, 114
    jle .p1_b
    cmp ebx, 382
    jge .p1_b
    cmp esi, 82
    jle .p1_b
    cmp esi, 278
    jge .p1_b
    mov eax, 0x00131a1d
    ; Header band: y in [82..110]
    cmp esi, 110
    jge .p1_check_status
    mov eax, 0x00161e22
    jmp .L5
.p1_check_status:
    ; "WASM_SANDBOX: SECURE" green status dot at (130, 250) r=4
    mov ecx, ebx
    sub ecx, 130
    imul ecx, ecx
    mov edx, esi
    sub edx, 250
    imul edx, edx
    add ecx, edx
    cmp ecx, 16
    jg .L5
    mov eax, 0x0034d399     ; emerald-400
    jmp .L5
.p1_b:
    mov eax, 0x00003a45

.L5:
    ; ----- Layer 5: Swarm Visualizer panel (480..768, 80..480) -----
    cmp ebx, 480
    jl .L6
    cmp ebx, 768
    jge .L6
    cmp esi, 80
    jl .L6
    cmp esi, 480
    jge .L6
    cmp ebx, 482
    jle .p2_b
    cmp ebx, 766
    jge .p2_b
    cmp esi, 82
    jle .p2_b
    cmp esi, 478
    jge .p2_b
    mov eax, 0x00131a1d
    cmp esi, 110
    jge .p2_marker
    mov eax, 0x00161e22
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
    mov eax, 0x00003a45

.L6:
    ; ----- Layer 6: Evolution Log panel (200..600, 410..500) -----
    cmp ebx, 200
    jl .L7
    cmp ebx, 600
    jge .L7
    cmp esi, 410
    jl .L7
    cmp esi, 500
    jge .L7
    cmp ebx, 202
    jle .p3_b
    cmp ebx, 598
    jge .p3_b
    cmp esi, 412
    jle .p3_b
    cmp esi, 498
    jge .p3_b
    mov eax, 0x00131a1d
    ; Header strip y in [412..432]
    cmp esi, 432
    jge .p3_lines
    mov eax, 0x00161e22
    jmp .L7
.p3_lines:
    ; Faux log lines: thin cyan ticks at evenly spaced y rows
    ; Row positions: 442, 452, 462, 472, 482 (5 rows, 10px apart)
    ; Tick at x in [210..230] indicates timestamp gutter
    mov ecx, esi
    sub ecx, 442
    cmp ecx, 0
    jl .L7
    test ecx, 9             ; close to a row boundary
    jnz .L7
    cmp ebx, 210
    jl .L7
    cmp ebx, 230
    jge .L7
    mov eax, 0x0000daf3
    jmp .L7
.p3_b:
    mov eax, 0x00003a45

.L7:
    ; ----- Layer 7: Thought Core (center 400, 280) -----
    mov ecx, ebx
    sub ecx, 400
    imul ecx, ecx
    mov edx, esi
    sub edx, 280
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
    mov eax, 0x0080e8f5     ; hot inner orb
    jmp .L8
.core_outer_orb:
    mov eax, 0x000a3a45     ; cool outer orb
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
    mov eax, 0x00111f24
    jmp .L8
.core_border:
    mov eax, 0x0000daf3
    jmp .L8
.core_halo:
    ; Additive halo tint over whatever Layer<7 painted
    add eax, 0x00050a0c

.L8:
    ; ----- Layer 8: Animated scanline across core -----
    ; scan_y = 152 + ((ebp>>1) & 255)  -> sweeps 152..407
    mov ecx, ebp
    shr ecx, 1
    and ecx, 255
    add ecx, 152
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
    mov eax, 0x00b0f5ff

.L9:
    ; ----- Layer 9a: Bottom nav pill (280..520, 550..590) -----
    cmp ebx, 280
    jl .L9_voice
    cmp ebx, 520
    jge .L9_voice
    cmp esi, 550
    jl .L9_voice
    cmp esi, 590
    jge .L9_voice
    mov eax, 0x00131a1d
    ; Top border
    cmp esi, 552
    jl .pill_b
    ; Inner dots: 5 nav icons evenly spaced (x cells: 300,360,400,460,500)
    ; Skip — voice will overlap center
    jmp .L9_voice
.pill_b:
    mov eax, 0x00003a45

.L9_voice:
    ; ----- Layer 9b: Voice button (cx=400, cy=540, r=32) -----
    mov ecx, ebx
    sub ecx, 400
    imul ecx, ecx
    mov edx, esi
    sub edx, 540
    imul edx, edx
    add ecx, edx
    cmp ecx, 1024           ; r > 32 -> outside
    jg .L10
    cmp ecx, 784            ; r in [28..32] -> ring
    jge .voice_ring
    mov eax, 0x0000daf3     ; solid cyan core
    jmp .L10
.voice_ring:
    mov eax, 0x00b0f5ff

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
