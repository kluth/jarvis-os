; ==========================================================
; JARVIS OS - Sovereign Minimal Bootloader (Rev 7.0)
; ----------------------------------------------------------
; This is a minimal Stage-1 loader. Its ONLY job is to:
; 1. Set VBE mode 0x11b (1280x1024x24).
; 2. Enter 32-bit Protected Mode.
; 3. Jump to the JRV-compiled Kernel Entry Point.
; ALL UI rendering and logic is handled by pure JRV.
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

    ; 1. Query and Set VBE 1280x1024x24 (LFB)
    mov ax, 0x4f01
    mov cx, 0x11b
    mov di, 0x9000
    int 0x10

    mov ax, 0x4f02
    mov bx, 0x411b
    int 0x10

    ; 2. Enter Protected Mode
    cli
    in al, 0x92
    or al, 2
    out 0x92, al
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:kernel_entry

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
kernel_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    ; 3. Jump to the JRV Kernel (Assuming loaded at 0x10000 by jrvc payload)
    ; In a real deployment, jrvc generates the machine code.
    jmp 0x10000

times 16896-($-$$) db 0
