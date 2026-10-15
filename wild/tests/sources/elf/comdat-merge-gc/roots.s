.section .rodata.live,"aMSG",@progbits,1,live,comdat
.globl live_string
live_string:
  .asciz "live string"
.section .data.live,"awG",@progbits,live,comdat
.globl live_companion
live_companion:
  .long 17

.section .rodata.retained,"aMSGR",@progbits,1,retained,comdat
.asciz "retained string"
.section .data.retained,"awG",@progbits,retained,comdat
.globl retained_companion
retained_companion:
  .long 19

.section .rodata.kept,"aMSG",@progbits,1,kept,comdat
.asciz "kept string"
.section .data.kept,"awG",@progbits,kept,comdat
.globl kept_companion
kept_companion:
  .long 23

.section .note.GNU-stack,"",@progbits
