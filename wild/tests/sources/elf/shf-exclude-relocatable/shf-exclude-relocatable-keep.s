.section .text.startup, "ax", @progbits
.globl keep_fn
.type keep_fn, @function
keep_fn:
  .dc.a keep_fn
  .byte 0
.size keep_fn, .-keep_fn

.section .data.excl, "awe", @progbits
.globl excl_var
.type excl_var, @object
.size excl_var, 4
excl_var:
  .long 37

.section .exclude, "e", @progbits
.ascii "ShouldBeExcluded"

.section .note.GNU-stack, "", @progbits
