//#Arch:ppc64le
//#Compiler:clang
//#LinkerDriver:gcc
//#DiffEnabled:false
//#Shared:defs.s
//#CompSoArgs:-fPIC
//#CompArgs:-fPIE
//#LinkArgs:-pie -Wl,-z,now

.abiversion 2
.text
.globl main
.type main, @function
main:
	addis 2, 12, .TOC.-main@ha
	addi 2, 2, .TOC.-main@l
	mflr 0
	std 0, 16(1)
	stdu 1, -64(1)
	std 31, 32(1)
	li 31, 0
	.macro call_one
	bl f\+
	nop
	add 31, 31, 3
	.endm
	.rept 5000
	call_one
	.endr
	cmpdi 31, 5000
	li 3, 42
	beq 0, 1f
	li 3, 1
1:
	ld 31, 32(1)
	addi 1, 1, 64
	ld 0, 16(1)
	mtlr 0
	blr
.size main, .-main

.section .note.GNU-stack,"",@progbits
