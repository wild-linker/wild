// Test that a dynamic R_X86_64_SIZE32 relocation cannot be emitted into .text.
//#Arch:x86_64
//#ReferenceLinkers:lld
//#LinkArgs:--no-gc-sections -shared
//#ExpectError:R_X86_64_SIZE32
//#ExpectErrorWild:Cannot apply dynamic relocation R_X86_64_SIZE32 to read-only section for symbol `foo`

.globl foo
.type foo,@object
.size foo,26
foo:
    .zero 26

.text
.globl _start
_start:
    movl $foo@SIZE, %eax
