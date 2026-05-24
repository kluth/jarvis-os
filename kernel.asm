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
    xor esi, esi            ; y

.y_loop:
    xor ebx, ebx            ; x

.x_loop:
    ; This code is the MACHINE LOWERING of the JRV UI modules.
    mov eax, 0x00131313     ; Background
    
    ; Telemetry: Active Agent Count (0x8000)
    ; Telemetry: Energy Usage (0x800C)
    
    ; Thought Core
    mov ecx, ebx
    sub ecx, 640
    imul ecx, ecx
    mov edx, esi
    sub edx, 512
    imul edx, edx
    add ecx, edx
    cmp ecx, 16384
    jg .panels
    mov eax, 0x00fff5c3
    add eax, [0x800C]       ; Dynamic life!
    jmp .draw

.panels:
    ; Health Panel (Dynamic violation bars)
    cmp ebx, 128
    jl .draw
    cmp ebx, 448
    jge .draw
    cmp esi, 128
    jl .draw
    cmp esi, 448
    jge .draw
    
    mov ecx, [0x8004]       ; Violation count
    imul ecx, 16
    mov edx, 448
    sub edx, ecx
    cmp esi, edx
    jl .p_bg
    mov eax, 0x00b4abff     ; Error red
    jmp .draw
.p_bg:
    mov eax, 0x00131313

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

    ; VSync
    mov edx, 0x3DA
.wait:
    in al, dx
    test al, 8
    jz .wait
    
    ; Simulate a background process updating telemetry
    inc dword [0x800C]
    and dword [0x800C], 0xFFFF
    
    jmp render_loop
