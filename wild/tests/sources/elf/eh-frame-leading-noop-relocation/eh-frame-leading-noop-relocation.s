//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:clang
//#LinkArgs:-no-pie --eh-frame-hdr --no-gc-sections

//#Config:plain:default
//#ReferenceLinkers:bfd,lld
//#ExpectSectionBytes:.eh_frame_hdr=0x01000000 8..12

//#Config:leading-noop:default
//#CompArgs:-Wa,--defsym,LEADING_NOOP=1
// BFD asserts on this input; LLD silently discards the live FDE.
//#ReferenceLinkers:
//#ExpectError:Unsupported .eh_frame relocation R_X86_64_NONE at 0x18 before FDE pc-begin
//#ExpectError:eh-frame-leading-noop-relocation.*\.o

//#Config:leading-noop-crel:leading-noop
//#CompArgs:-Wa,--crel,--allow-experimental-crel
//#RequiresCompilerFlags:-Wa,--crel,--allow-experimental-crel

.section .text.start,"ax",@progbits
.balign 4
.globl _start
_start:
    mov $60, %eax
    mov $42, %edi
    syscall
.Lend:

.section .eh_frame,"a",@progbits
.Lcie:
    .long .Lcie_end - .Lcie - 4
    .long 0
    .byte 1
    .asciz "zR"
    .uleb128 1
    .sleb128 -8
    .byte 16
    .uleb128 1
    .byte 0x1b
    .byte 0x0c, 7, 8, 0x90, 1
    .balign 8, 0
.Lcie_end:
.Lfde:
    .long .Lfde_end - .Lfde - 4
    .long . - .Lcie
.ifdef LEADING_NOOP
    .reloc .Lfde, R_X86_64_NONE, 0
.endif
    .long _start - .
    .long .Lend - _start
    .uleb128 0
    .balign 8, 0
.Lfde_end:

.section .note.GNU-stack,"",@progbits
