.abiversion 2
.text
.macro def_one
.globl f\+
.type f\+, @function
f\+:
	li 3, 1
	blr
.size f\+, .-f\+
.endm
.rept 5000
def_one
.endr

.section .note.GNU-stack,"",@progbits
