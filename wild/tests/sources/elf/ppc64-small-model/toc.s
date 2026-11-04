.abiversion 2

.section .toc,"aw",@progbits
.align 3
.globl payload
payload:
  .quad 42

.text
.globl load_toc16_ds
.type load_toc16_ds, @function
load_toc16_ds:
  addis 2, 12, .TOC.-load_toc16_ds@ha
  addi 2, 2, .TOC.-load_toc16_ds@l
  .localentry load_toc16_ds, .-load_toc16_ds
  ld 3, payload@toc(2)
  blr
.size load_toc16_ds, .-load_toc16_ds

.globl load_toc16
.type load_toc16, @function
load_toc16:
  addis 2, 12, .TOC.-load_toc16@ha
  addi 2, 2, .TOC.-load_toc16@l
  .localentry load_toc16, .-load_toc16
  addi 3, 2, payload@toc
  ld 3, 0(3)
  blr
.size load_toc16, .-load_toc16

.section .note.GNU-stack,"",@progbits
