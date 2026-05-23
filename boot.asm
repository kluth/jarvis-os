; ==========================================================
; JARVIS OS - Aetheris Spatial Interface v5 (OMEGA-ULTRA)
; ----------------------------------------------------------
; Resolution: 1024x768x24.
; Features: Full spatial composition, distance-glows,
;           rounded-glass, typography skeletons.
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

    mov ax, 0x4f02
    mov bx, 0x4118
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
    ; 1. Deep Obsidian foundation
    mov eax, 0x00131313

    ; 2. Spatial Grid lattice (Subtle)
    test ebx, 63
    jz .grid_hit
    test esi, 63
    jnz .L2
.grid_hit:
    mov eax, 0x00161a1d

.L2:
    ; 3. Top Header Bar (Glass)
    cmp esi, 48
    jge .L3
    cmp esi, 46
    jge .hdr_border
    mov eax, 0x000e0e0e
    ; Header Logo skeleton
    cmp esi, 18
    jl .L3
    cmp esi, 30
    jge .L3
    cmp ebx, 480
    jl .L3
    cmp ebx, 544
    jge .L3
    mov eax, 0x00fff5c3
    jmp .L3
.hdr_border:
    mov eax, 0x004c493b

.L3:
    ; 4. Left Side Rail (0..48, 200..600)
    cmp ebx, 48
    jge .L4
    cmp esi, 200
    jl .L4
    cmp esi, 600
    jge .L4
    cmp ebx, 46
    jge .rail_border
    mov eax, 0x000e0e0e
    jmp .L4
.rail_border:
    mov eax, 0x004c493b

.L4:
    ; 5. Substrate Health panel (80..336, 80..336)
    cmp ebx, 80
    jl .L5
    cmp ebx, 336
    jge .L5
    cmp esi, 80
    jl .L5
    cmp esi, 336
    jge .L5
    ; Border check
    cmp ebx, 81
    jle .p1_b
    cmp ebx, 335
    jge .p1_b
    cmp esi, 81
    jle .p1_b
    cmp esi, 335
    jge .p1_b
    ; Header band
    cmp esi, 112
    jge .p1_c
    mov eax, 0x00201f1f
    jmp .L5
.p1_c:
    mov eax, 0x001b1b1c
    jmp .L5
.p1_b:
    mov eax, 0x004c493b

.L5:
    ; 6. Swarm Visualizer (688..944, 80..400)
    cmp ebx, 688
    jl .L6
    cmp ebx, 944
    jge .L6
    cmp esi, 80
    jl .L6
    cmp esi, 400
    jge .L6
    cmp ebx, 689
    jle .p2_b
    cmp ebx, 943
    jge .p2_b
    cmp esi, 81
    jle .p2_b
    cmp esi, 399
    jge .p2_b
    cmp esi, 112
    jge .p2_c
    mov eax, 0x00201f1f
    jmp .L6
.p2_c:
    mov eax, 0x001b1b1c
    jmp .L6
.p2_b:
    mov eax, 0x004c493b

.L6:
    ; 7. Evolution Log (300..724, 600..720)
    cmp ebx, 300
    jl .L7
    cmp ebx, 724
    jge .L7
    cmp esi, 600
    jl .L7
    cmp esi, 720
    jge .L7
    cmp ebx, 301
    jle .p3_b
    cmp ebx, 723
    jge .p3_b
    cmp esi, 601
    jle .p3_b
    cmp esi, 719
    jge .p3_b
    cmp esi, 620
    jge .p3_c
    mov eax, 0x00201f1f
    jmp .L7
.p3_c:
    mov eax, 0x001b1b1c
    jmp .L7
.p3_b:
    mov eax, 0x004c493b

.L7:
    ; 8. Thought Core (512, 384)
    mov ecx, ebx
    sub ecx, 512
    imul ecx, ecx
    mov edx, esi
    sub edx, 384
    imul edx, edx
    add ecx, edx

    cmp ecx, 65536
    jg .L8
    
    ; Additive distance-based halo
    mov edx, ecx
    shr edx, 8
    neg edx
    add edx, 256
    cmp edx, 0
    jl .core_body
    shr edx, 4
    shl edx, 1
    add eax, edx
    
.core_body:
    cmp ecx, 16384
    jg .L8
    cmp ecx, 14400
    jge .core_border
    mov eax, 0x001b1b1c
    ; Pulsing Inner Orb
    mov edx, ebp
    shr edx, 2
    and edx, 31
    add edx, 64
    imul edx, edx
    cmp ecx, edx
    jg .L8
    mov eax, 0x00fff09c
    jmp .L8
.core_border:
    mov eax, 0x00f3da00

.L8:
    ; 9. Bottom Nav Pill
    cmp ebx, 440
    jl .L9
    cmp ebx, 584
    jge .L9
    cmp esi, 720
    jl .L9
    cmp esi, 752
    jge .L9
    mov eax, 0x001b1b1c
    cmp esi, 722
    jge .L9
    mov eax, 0x004c493b

.L9:
    ; Draw & Next
    stosw
    shr eax, 16
    stosb
    inc ebx
    cmp ebx, 1024
    jl .x_loop
    inc esi
    cmp esi, 768
    jl .y_loop

    mov edx, 0x3DA
.wait_retrace:
    in al, dx
    test al, 8
    jz .wait_retrace
    jmp render_loop

times 16896-($-$$) db 0
