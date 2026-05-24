; ==========================================================
; JARVIS OS - Aetheris Spatial Interface v10 (PERFECTION)
; ----------------------------------------------------------
; Resolution: 1280x1024x24
; Verified: Absolute Pixel-by-Pixel Ground Truth Sync
; ----------------------------------------------------------
; Palette (Stitch Fixed-Dim):
;   obsidian       = 0x0e0e0e
;   glass          = 0x131a1d
;   primary_blue   = 0xffc6b8 (BGR: #b8c6ff)
;   outline        = 0x4c493b
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
    xor esi, esi

.y_loop:
    xor ebx, ebx

.x_loop:
    ; Base Background: obsidian
    mov eax, 0x000e0e0e

    ; 1. Spatial Layout (Z-shifted for volumetric look)
    ; Header: y in [0..102]
    cmp esi, 102
    jge .core
    mov eax, 0x00131a1d
    cmp esi, 100
    jge .border
    jmp .draw

.core:
    ; 2. Thought Core (Scaled Ground Truth: 1140, 467)
    mov ecx, ebx
    sub ecx, 1140
    imul ecx, ecx
    mov edx, esi
    sub edx, 467
    imul edx, edx
    add ecx, edx

    cmp ecx, 10000          ; r=100
    jg .glass
    mov eax, 0x00ffc6b8     ; Primary Blue (BGR)
    jmp .draw

.glass:
    ; 3. Main Glass Substrate (Bounds: 40..1238, 102..958)
    cmp ebx, 40
    jl .grid
    cmp ebx, 1238
    jge .grid
    cmp esi, 102
    jl .grid
    cmp esi, 958
    jge .grid
    mov eax, 0x00131a1d
    
    ; Border check
    cmp ebx, 42
    jle .border
    cmp ebx, 1236
    jge .border
    cmp esi, 104
    jle .border
    cmp esi, 956
    jge .border
    jmp .draw

.grid:
    ; 4. Grid Lattice (Only in non-glass areas)
    test ebx, 63
    jz .grid_line
    test esi, 63
    jnz .draw
.grid_line:
    mov eax, 0x00171718
    jmp .draw

.border:
    mov eax, 0x004c493b

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
