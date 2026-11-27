# Stand-in for a crtbegin.o from GCC built without .init_array support: the start markers.
  .section .ctors,"aw",@progbits
  .p2align 3
  .quad -1
  .section .dtors,"aw",@progbits
  .p2align 3
  .quad -1
  .section .note.GNU-stack,"",@progbits
