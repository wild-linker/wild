.data
.globl value
.type value,@object
.size value,4
value:
    .long 42

.text
.globl function
.type function,@function
function:
    mov $42, %eax
    ret
.size function,.-function

.section .note.GNU-stack,"",@progbits
