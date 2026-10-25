//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-Wa,-mrelax-relocations=no
//#LinkArgs:-z noexecstack
//#NoSection:.got
//#LinkArgs:-no-pie

.text
.globl _start
_start:
    // A relaxed full-width load must not leave an unused GOT entry.
    mov target@GOTPCREL(%rip), %rax
    mov (%rax), %edi
    mov $60, %eax
    syscall

.data
.hidden target
target:
    .long 42

.section .note.GNU-stack,"",@progbits
