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
    mov edi, [0x9028]       ; LFB from VBE block
    test edi, edi
    jnz .start_render
    mov edi, 0xFD000000     ; FALLBACK for QEMU std-vga

.start_render:
    xor esi, esi            ; y
.y_loop:
    xor ebx, ebx            ; x
.x_loop:
    ; Background
    mov eax, 0x00131313
    
    ; Thought Core (Centered 512, 384 for 1024x768)
    mov ecx, ebx
    sub ecx, 512
    imul ecx, ecx
    mov edx, esi
    sub edx, 384
    imul edx, edx
    add ecx, edx
    cmp ecx, 16384          ; r=128
    jg .draw
    mov eax, 0x00fff5c3
    
.draw:
    stosw
    shr eax, 16
    stosb
    inc ebx
    cmp ebx, 1024
    jl .x_loop
    inc esi
    cmp esi, 768
    jl .y_loop

    ; VSync wait
    mov edx, 0x3DA
.wait:
    in al, dx
    test al, 8
    jz .wait
    jmp render_loop

