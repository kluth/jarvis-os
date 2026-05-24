[bits 32]
[org 0x10000]

kernel_main:
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
    test edi, edi
    jz .hang

    xor esi, esi            ; y
.y_loop:
    xor ebx, ebx            ; x
.x_loop:
    ; Background
    mov eax, 0x00131313
    
    ; Thought Core (Centered 400, 300 for 800x600)
    mov ecx, ebx
    sub ecx, 400
    imul ecx, ecx
    mov edx, esi
    sub edx, 300
    imul edx, edx
    add ecx, edx
    cmp ecx, 10000
    jg .draw
    mov eax, 0x00fff5c3
    
.draw:
    stosw
    shr eax, 16
    stosb
    inc ebx
    cmp ebx, 800
    jl .x_loop
    inc esi
    cmp esi, 600
    jl .y_loop

    ; VSync wait
    mov edx, 0x3DA
.wait:
    in al, dx
    test al, 8
    jz .wait
    jmp render_loop

.hang:
    jmp $
