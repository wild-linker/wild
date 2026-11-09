.section .text.startup, "axe", @progbits
.globl drop_fn
.type drop_fn, @function
drop_fn:
  .dc.a drop_fn
  .byte 0
.size drop_fn, .-drop_fn

.section .note.GNU-stack, "", @progbits
