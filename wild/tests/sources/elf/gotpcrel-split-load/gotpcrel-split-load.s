// Loading GOT+4 reads the upper half of a pointer, not symbol_address+4.
//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-Wa,-mrelax-relocations=no
//#LinkArgs:-z noexecstack
//#LinkArgs:-no-pie

.text
.globl _start
_start:
    // Reference: the upper 32 bits of the complete pointer.
    mov target@GOTPCREL(%rip), %rax
    shr $32, %rax

    // The same bits loaded directly from the upper half of the GOT slot.
    // This produces R_X86_64_GOTPCREL with addend 0 instead of -4.
    mov target@GOTPCREL+4(%rip), %edx
    cmp %eax, %edx
    jne failed

    mov $42, %edi
    jmp exit
failed:
    mov $43, %edi
exit:
    mov $60, %eax
    syscall

.data
.globl target
.hidden target
target:
    .quad 0

.section .note.GNU-stack,"",@progbits
