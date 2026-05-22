[org 0x7c00]
[bits 16]

start:
    ; Set up segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00

    ; 1. Enter VGA Graphics Mode (320x200, 256 colors)
    mov ax, 0x0013
    int 0x10

    ; 2. Draw Holographic Substrate (Background)
    ; Fill screen with JARVIS Blue (Color 1)
    mov ax, 0xa000
    mov es, ax
    xor di, di
    mov al, 1       ; Blue
    mov cx, 64000   ; 320 * 200
    rep stosb

    ; 3. Draw "Spatial Grid" (Simulated 3D projection)
    mov di, 320 * 100
    mov al, 15      ; White pixels
    mov cx, 320
    rep stosb       ; Horizontal horizon line

    ; 4. Print Status to Serial (for Telemetry)
    mov si, msg_boot
    call print_serial

    ; 5. Print status in Graphics Mode (Visual)
    mov bh, 0
    mov dh, 1       ; Row
    mov dl, 1       ; Column
    mov ah, 0x02    ; Set cursor
    int 0x10

    mov si, msg_gui_ready
    call print_vga_text

    ; Halt
    jmp $

print_serial:
.loop:
    lodsb
    or al, al
    jz .done
    mov dx, 0x3f8
    out dx, al
    jmp .loop
.done:
    ret

print_vga_text:
.loop:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0e
    mov bl, 15      ; White text
    int 0x10
    jmp .loop
.done:
    ret

msg_boot db 'JARVIS OS v0.1.2 (AETHERIS) - Holographic Active', 10, 13, 0
msg_gui_ready db 'AETHERIS SPATIAL INTERFACE: V1.0 ACTIVE', 0

times 510-($-$$) db 0
dw 0xaa55
