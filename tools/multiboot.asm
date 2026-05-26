; ==========================================================
; JARVIS OS - Multiboot v1 Boot Stub
; ----------------------------------------------------------
; Converts the JRV kernel ELF into a QEMU-bootable image
; by prepending a standard Multiboot v1 header.
;
; The JRV compiler produces an ELF64 that GRUB/QEMU can
; boot directly IF the Multiboot header is present. Since
; the JRV toolchain doesn't yet emit the header, we attach
; it here for CI integration testing.
;
; Once jrvc supports the --multiboot flag, this file becomes
; redundant and can be retired.
; ==========================================================

[BITS 32]

; ----------------------------------------------------------
; Multiboot v1 Header
; ----------------------------------------------------------
section .multiboot_header
align 4

MBOOT_MAGIC    equ 0x1BADB002
MBOOT_FLAGS    equ 0x00000003  ; bit0=module_align, bit1=memory_map
MBOOT_CHECKSUM equ -(MBOOT_MAGIC + MBOOT_FLAGS)

; Multiboot v1 header structure
dd MBOOT_MAGIC
dd MBOOT_FLAGS
dd MBOOT_CHECKSUM

; For flags=0x3, the header must be 12 bytes total.
; No aout_kludge, no video mode info needed.

; ----------------------------------------------------------
; Entry Point
; ----------------------------------------------------------
section .text
global _start
_start:
    ; The bootloader (GRUB/QEMU -kernel) has already set up:
    ;   - Protected mode (32-bit)
    ;   - A20 gate enabled
    ;   - Interrupts disabled
    ;   - EAX = Multiboot magic (0x2BADB002)
    ;   - EBX = Multiboot info structure address

    ; Set up a minimal stack
    mov esp, stack_top

    ; Push Multiboot info pointer for kernel
    push ebx          ; boot_info
    push eax          ; magic

    ; Jump to the Rust/JRV kernel entry point.
    ; The kernel is linked at its own address via the ELF.
    ; If jrvc emits the entry at a specific symbol, call it here.
    ; For now, we call the multiboot entry convention:
    ;   extern rust_main(magic: u32, boot_info: &BootInfo)
    extern kernel_main
    call kernel_main

    ; Should never return — halt if it does
    cli
.halt:
    hlt
    jmp .halt

; ----------------------------------------------------------
; Stack (16KB)
; ----------------------------------------------------------
section .bss
align 16
stack_bottom:
    resb 16384
stack_top:
