; ==========================================================
; JARVIS OS - Aetheris Spatial Interface v15 (PIXEL-PERFECT)
; ----------------------------------------------------------
; Resolution: 1280x1024x24
; Methodology: Formal CSS Specification Synchronization
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
    mov edi, [0x9028]       ; LFB
    xor esi, esi

.y_loop:
    xor ebx, ebx

.x_loop:
    ; 1. Surface Background (#131313)
    mov eax, 0x00131313

    ; 2. Spatial Grid (64px interval, #353534)
    test ebx, 63
    jz .grid_hit
    test esi, 63
    jnz .L2
.grid_hit:
    mov eax, 0x00343535

.L2:
    ; 3. Thought Core (Absolute Center: 640, 512, r=128)
    mov ecx, ebx
    sub ecx, 640
    imul ecx, ecx
    mov edx, esi
    sub edx, 512
    imul edx, edx
    add ecx, edx

    cmp ecx, 16384          ; r=128
    jg .panels
    
    ; Primary Glow (#c3f5ff)
    mov eax, 0x00fff5c3
    cmp ecx, 14400          ; r=120
    jge .draw
    mov eax, 0x00131313     ; Center substrate
    jmp .draw

.panels:
    ; 4. Glass Panels (#131313 with #3b494c outline)
    ; Health: 128..448, 128..448
    cmp ebx, 128
    jl .p_spectrogram
    cmp ebx, 448
    jge .p_spectrogram
    cmp esi, 128
    jl .p_spectrogram
    cmp esi, 448
    jge .p_spectrogram
    mov eax, 0x00131313
    cmp ebx, 130
    jle .p_border
    cmp ebx, 446
    jge .p_border
    cmp esi, 130
    jle .p_border
    cmp esi, 446
    jge .p_border
    jmp .draw

.p_spectrogram:
    ; Spectrogram: 832..1152, 576..896
    cmp ebx, 832
    jl .draw
    cmp ebx, 1152
    jge .draw
    cmp esi, 576
    jl .draw
    cmp esi, 896
    jge .draw
    mov eax, 0x00131313
    cmp ebx, 834
    jle .p_border
    cmp ebx, 1150
    jge .p_border
    cmp esi, 578
    jle .p_border
    cmp esi, 894
    jge .p_border
    jmp .draw

.p_border:
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
