	.globl foo
	.section .foo,"aRw"
	.p2align 16, 0x0
foo:

	.section .data.empty,"aw"
	.p2align 17
	.globl data_marker
data_marker:

	.section .data.live,"aw"
	.globl live_data
live_data:
	.byte 42

	.section .data.unused,"aw"
	.p2align 20
	.globl unused_marker
unused_marker:
