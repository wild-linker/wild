//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:clang
//#LinkArgs:-no-pie --eh-frame-hdr
// BFD cannot build the unwind index from unordered relocations.
//#ReferenceLinkers:lld

//#Config:across-frames:default
//#LinkArgs:--no-gc-sections
//#ExpectSectionBytes:.eh_frame_hdr=0x03000000 8..12

//#Config:within-frame:across-frames
//#CompArgs:-Wa,--defsym,WITHIN_FRAME=1

//#Config:gc:default
//#LinkArgs:--gc-sections
//#ExpectSectionBytes:.eh_frame_hdr=0x02000000 8..12
// TODO: Enable lld once CI consistently has lld 22 or later
//#ReferenceLinkers:

//#Config:crel:gc
//#CompArgs:-Wa,--crel,--allow-experimental-crel
//#RequiresCompilerFlags:-Wa,--crel,--allow-experimental-crel

.section .text.start,"ax",@progbits
.balign 4
.globl _start
_start:
    call other
    mov $60, %eax
    mov $42, %edi
    syscall
.Lstart_end:

.section .text.other,"ax",@progbits
other:
    ret
.Lother_end:

.section .text.unused,"ax",@progbits
unused:
    ret
.Lunused_end:

.section .gcc_except_table,"a",@progbits
.Llsda:
    .byte 0xff, 0xff, 1, 0

.section .eh_frame,"a",@progbits
.Lcie:
    .long .Lcie_end - .Lcie - 4
    .long 0
    .byte 1
    .asciz "zLR"
    .uleb128 1
    .sleb128 -8
    .byte 16
    .uleb128 2
    .byte 0x1b, 0x1b
    .byte 0x0c, 7, 8, 0x90, 1
    .balign 8, 0
.Lcie_end:

.macro frame name, end
.Lfde_\name:
    .long .Lfde_end_\name - .Lfde_\name - 4
    .long . - .Lcie
.Lpc_\name:
    .long 0
    .long \end - \name
    .uleb128 4
.Llsda_\name:
    .long 0
    .byte 0, 0, 0
.Lfde_end_\name:
.endm

frame _start, .Lstart_end
frame other, .Lother_end
frame unused, .Lunused_end

.macro relocations name
.ifdef WITHIN_FRAME
    .reloc .Llsda_\name, R_X86_64_PC32, .Llsda
    .reloc .Lpc_\name, R_X86_64_PC32, \name
.else
    .reloc .Lpc_\name, R_X86_64_PC32, \name
    .reloc .Llsda_\name, R_X86_64_PC32, .Llsda
.endif
.endm

.ifdef WITHIN_FRAME
relocations _start
relocations other
relocations unused
.else
relocations unused
relocations other
relocations _start
.endif

.section .note.GNU-stack,"",@progbits
