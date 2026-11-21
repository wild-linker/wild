.section .idata$2,"dr"
.globl _head_libfoo_a
_head_libfoo_a:
.rva .idata$4
.long 0
.long 0
.rva __lib64_foo_a_iname
.rva .idata$5

.section .idata$4,"dr"
.section .idata$5,"dr"
