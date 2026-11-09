.section .text, "ax", @progbits
.globl use_excl
.type use_excl, @function
use_excl:
  .dc.a excl_var
  .byte 0
.size use_excl, .-use_excl

.section .note.GNU-stack, "", @progbits
