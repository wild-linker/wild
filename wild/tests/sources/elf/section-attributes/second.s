.section .rodata.cst8,"aM",@progbits,8
.quad 42

.section uniform_strings,"aMS",@progbits,1
.asciz "second"
.section uniform_constants,"aM",@progbits,8
.quad 2
.section mixed_sizes,"aM",@progbits,8
.quad 2

.section .note.GNU-stack,"",@progbits
