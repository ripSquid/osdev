global long_mode_start

section .text
bits 64
long_mode_start:

    mov ax, 0
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mod ds, ax
    moc es, ax


    mov dword [0xb8000], 0x2f4b2f4f
    hlt