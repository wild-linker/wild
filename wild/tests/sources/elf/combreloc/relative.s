.section .data.rel.local,"aw",@progbits
.p2align 3
.globl relative_pointers
.hidden relative_pointers
.type relative_pointers,@object
relative_pointers:
    .quad 0, 0
    .reloc relative_pointers+8, R_X86_64_64, relative_target
    .reloc relative_pointers, R_X86_64_64, relative_target
.size relative_pointers, .-relative_pointers

.section .rodata,"a",@progbits
.p2align 2
relative_target:
    .long 31

.section .note.GNU-stack,"",@progbits
