; ==========================================================
; JARVIS OS - Sovereign Minimal Bootloader (Rev 7.1)
; ----------------------------------------------------------
; Hardened loader with drive-ID persistence and VBE retry.
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

    ; 2. Set VBE mode 0x115 (800x600x24) for max compatibility
    mov ax, 0x4f01
    mov cx, 0x115
    mov di, 0x9000
    int 0x10
    cmp ax, 0x004f
    jne .vbe_error

    mov ax, 0x4f02
    mov bx, 0x4115          ; 0x115 + LFB bit
    int 0x10
    cmp ax, 0x004f
    jne .vbe_error

    ; 3. Load the JRV Kernel (Sector 2 onwards)
    mov ax, 0x1000
    mov es, ax
    xor bx, bx
    
    mov ah, 0x02
    mov al, 64              ; Read 64 sectors (32KB)
    mov ch, 0
    mov cl, 2               ; Sector 2
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    jc .disk_error

    ; 4. Enter Protected Mode
    cli
    in al, 0x92
    or al, 2
    out 0x92, al
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:kernel_entry_jump

.vbe_error:
    mov ah, 0x0e
    mov al, 'V'
    int 0x10
    jmp $

.disk_error:
    mov ah, 0x0e
    mov al, 'D'
    int 0x10
    jmp $

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

[bits 32]
kernel_entry_jump:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    ; Check if LFB address was populated
    mov eax, [0x9028]
    test eax, eax
    jz .no_lfb

    ; Jump to kernel
    jmp 0x10000

.no_lfb:
    jmp $
