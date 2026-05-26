; ==========================================================
; JARVIS OS - Aetheris Spatial Renderer (Stage-1 & 2)
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

    ; 1. Save Drive ID
    mov [boot_drive], dl

    ; 2. Load Stage-2 (The rest of this binary, 32 sectors)
    mov ax, 0x07e0 ; Right after Stage-1 (0x7c00 + 512 = 0x7e00)
    mov es, ax
    xor bx, bx
    
    mov ah, 0x02
    mov al, 32              ; Read 32 sectors
    mov ch, 0
    mov cl, 2               ; Start from Sector 2
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13

    ; 3. Set VBE mode 0x411B (1280x1024x32)
    mov ax, 0x4f02
    mov bx, 0x411b
    int 0x10

    ; 4. Enter Protected Mode
    cli
    in al, 0x92
    or al, 2
    out 0x92, al
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:render_loop_start

boot_drive: db 0

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

; --- STAGE 2 (Starting at offset 512) ---
[bits 32]
render_loop_start:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    ; Infinite Rendering Loop
.loop:
    mov edi, 0xFD000000 ; LFB Base
    xor esi, esi        ; Y counter
.y_loop:
    xor ebx, ebx        ; X counter
.x_loop:
    ; Call auto-generated pixel logic
    call pixel_logic

    ; Draw pixel
    mov [edi], eax
    add edi, 4

    inc ebx
    cmp ebx, 1280
    jl .x_loop

    inc esi
    cmp esi, 1024
    jl .y_loop

    ; Collaborative Pulse (wait a bit)
    mov ecx, 1000000
.wait:
    dec ecx
    jnz .wait

    jmp .loop

%include "boot_v11.asm"
