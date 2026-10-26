.section .rodata.plain,"a",@progbits
.byte 42
.section .rodata.str1.1,"aMS",@progbits,1
.asciz "hello"

.section uniform_strings,"aMS",@progbits,1
.asciz "first"
.section uniform_constants,"aM",@progbits,8
.quad 1
.section mixed_sizes,"aM",@progbits,1
.byte 1

.section .note.GNU-stack,"",@progbits
