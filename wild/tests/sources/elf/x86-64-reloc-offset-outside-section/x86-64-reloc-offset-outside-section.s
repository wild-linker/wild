// Checks that a relocation whose offset is past the end of its section is reported as an error,
// rather than causing a panic.
// lld accepts this input, so only GNU ld is used as a reference.
//#Arch:x86_64
//#Compiler:clang
//#LinkArgs:-static
//#ReferenceLinkers:bfd
//#ExpectError:offset.*0x1000

.section .text,"ax",@progbits
.globl _start
_start:
  .quad 0
  .reloc _start + 4096, R_X86_64_64, target

.section .data,"aw",@progbits
.globl target
target:
  .quad 1
