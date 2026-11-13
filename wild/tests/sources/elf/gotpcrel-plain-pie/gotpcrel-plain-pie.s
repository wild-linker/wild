//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-Wa,-mrelax-relocations=no
//#Mode:dynamic
//#LinkArgs:-pie -z noexecstack

// rustc emits only plain R_X86_64_GOTPCREL. In a PIE, wild and GNU ld relax the mov to a lea
// because `target` is hidden, whereas LLD keeps the GOT load. Both load 42.

.text
.globl _start
.type _start, @function
_start:
    mov target@GOTPCREL(%rip), %rax
    mov (%rax), %edi
    mov $60, %eax
    syscall
.size _start, .-_start

.data
.hidden target
target:
    .long 42

.section .note.GNU-stack,"",@progbits
