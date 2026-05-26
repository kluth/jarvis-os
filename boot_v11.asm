; AUTO-GENERATED PERFECTION RENDERER
[bits 32]
pixel_logic:
    ; --- Thought Core ---
    mov ecx, ebx
    sub ecx, 428
    imul ecx, ecx
    mov edx, esi
    sub edx, 749
    imul edx, edx
    add ecx, edx
    cmp ecx, 2500 ; r=50 pulse
    jg .panels
    mov eax, 0x00ffc6b8
    jmp .draw
.panels:
    cmp esi, 64
    jge .row_64
    cmp ebx, 0
    jl .next_0_0
    cmp ebx, 64
    jge .next_0_0
    mov eax, 0x0022200f
    jmp .draw
.next_0_0:
    cmp ebx, 64
    jl .next_64_0
    cmp ebx, 128
    jge .next_64_0
    mov eax, 0x0016150f
    jmp .draw
.next_64_0:
    cmp ebx, 384
    jl .next_384_0
    cmp ebx, 448
    jge .next_384_0
    mov eax, 0x001a1911
    jmp .draw
.next_384_0:
    cmp ebx, 448
    jl .next_448_0
    cmp ebx, 512
    jge .next_448_0
    mov eax, 0x00141411
    jmp .draw
.next_448_0:
    jmp .row_64
.row_64:
    cmp esi, 128
    jge .row_128
    cmp ebx, 0
    jl .next_0_64
    cmp ebx, 64
    jge .next_0_64
    mov eax, 0x00242210
    jmp .draw
.next_0_64:
    cmp ebx, 64
    jl .next_64_64
    cmp ebx, 128
    jge .next_64_64
    mov eax, 0x00161610
    jmp .draw
.next_64_64:
    cmp ebx, 384
    jl .next_384_64
    cmp ebx, 448
    jge .next_384_64
    mov eax, 0x00171614
    jmp .draw
.next_384_64:
    jmp .row_128
.row_128:
    cmp esi, 192
    jge .row_192
    cmp ebx, 192
    jl .next_192_128
    cmp ebx, 256
    jge .next_192_128
    mov eax, 0x003d370a
    jmp .draw
.next_192_128:
    cmp ebx, 256
    jl .next_256_128
    cmp ebx, 320
    jge .next_256_128
    mov eax, 0x003b340a
    jmp .draw
.next_256_128:
    cmp ebx, 320
    jl .next_320_128
    cmp ebx, 384
    jge .next_320_128
    mov eax, 0x0013120e
    jmp .draw
.next_320_128:
    cmp ebx, 384
    jl .next_384_128
    cmp ebx, 448
    jge .next_384_128
    mov eax, 0x00222012
    jmp .draw
.next_384_128:
    cmp ebx, 448
    jl .next_448_128
    cmp ebx, 512
    jge .next_448_128
    mov eax, 0x00161610
    jmp .draw
.next_448_128:
    jmp .row_192
.row_192:
    cmp esi, 256
    jge .row_256
    cmp ebx, 128
    jl .next_128_192
    cmp ebx, 192
    jge .next_128_192
    mov eax, 0x0013120c
    jmp .draw
.next_128_192:
    cmp ebx, 192
    jl .next_192_192
    cmp ebx, 256
    jge .next_192_192
    mov eax, 0x00635308
    jmp .draw
.next_192_192:
    cmp ebx, 256
    jl .next_256_192
    cmp ebx, 320
    jge .next_256_192
    mov eax, 0x005d4d07
    jmp .draw
.next_256_192:
    cmp ebx, 320
    jl .next_320_192
    cmp ebx, 384
    jge .next_320_192
    mov eax, 0x0015130c
    jmp .draw
.next_320_192:
    jmp .row_256
.row_256:
    cmp esi, 320
    jge .row_320
    cmp ebx, 128
    jl .next_128_256
    cmp ebx, 192
    jge .next_128_256
    mov eax, 0x00181812
    jmp .draw
.next_128_256:
    cmp ebx, 192
    jl .next_192_256
    cmp ebx, 256
    jge .next_192_256
    mov eax, 0x00211f0f
    jmp .draw
.next_192_256:
    cmp ebx, 256
    jl .next_256_256
    cmp ebx, 320
    jge .next_256_256
    mov eax, 0x001b190d
    jmp .draw
.next_256_256:
    jmp .row_320
.row_320:
    cmp esi, 384
    jge .row_384
    cmp ebx, 128
    jl .next_128_320
    cmp ebx, 192
    jge .next_128_320
    mov eax, 0x00484214
    jmp .draw
.next_128_320:
    cmp ebx, 192
    jl .next_192_320
    cmp ebx, 256
    jge .next_192_320
    mov eax, 0x00292617
    jmp .draw
.next_192_320:
    cmp ebx, 256
    jl .next_256_320
    cmp ebx, 320
    jge .next_256_320
    mov eax, 0x0015140f
    jmp .draw
.next_256_320:
    jmp .row_384
.row_384:
    cmp esi, 448
    jge .row_448
    cmp ebx, 128
    jl .next_128_384
    cmp ebx, 192
    jge .next_128_384
    mov eax, 0x001a190c
    jmp .draw
.next_128_384:
    jmp .row_448
.row_448:
    cmp esi, 512
    jge .row_512
    jmp .row_512
.row_512:
    cmp esi, 576
    jge .row_576
    jmp .row_576
.row_576:
    cmp esi, 640
    jge .row_640
    jmp .row_640
.row_640:
    cmp esi, 704
    jge .row_704
    jmp .row_704
.row_704:
    cmp esi, 768
    jge .row_768
    jmp .row_768
.row_768:
    cmp esi, 832
    jge .row_832
    jmp .row_832
.row_832:
    cmp esi, 896
    jge .row_896
    jmp .row_896
.row_896:
    cmp esi, 960
    jge .row_960
    jmp .row_960
.row_960:
    cmp esi, 1024
    jge .row_1024
    jmp .row_1024
.row_1024:
.draw:
    ret
