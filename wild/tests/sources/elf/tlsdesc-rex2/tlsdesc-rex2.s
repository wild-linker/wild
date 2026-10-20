//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:gcc
//#LinkArgs:-z noexecstack
// LLD 21 rejects REX2 TLSDESC instructions targeting the lower registers.
//#ReferenceLinkers:bfd
//#RunEnabled:false

//#Config:local-exec:default
//#Object:tlsdesc-rex2-def.s
//#ExpectInstructions:.text 0 {rex2} mov $-8, %rax; xchg %ax, %ax; {rex2} mov $-8, %r8; mov %r8, %rax; xchg %ax, %ax; mov $-8, %r16; mov %r16, %rax; xchg %ax, %ax; mov $-8, %r24; mov %r24, %rax; xchg %ax, %ax; ret

//#Config:initial-exec:default
//#Mode:dynamic
//#Shared:tlsdesc-rex2-def.s
// Each linker builds its own shared-object input.
//#DiffIgnore:.dynamic.DT_NEEDED
//#ExpectSectionBytes:.text=0xd5088b05 0..4
//#ExpectSectionBytes:.text=0xd50c8b05 10..14
//#ExpectSectionBytes:.text=0xd5488b05 23..27
//#ExpectSectionBytes:.text=0xd54c8b05 37..41
//#ExpectInstructions:.text 8 xchg %ax, %ax
//#ExpectInstructions:.text 18 mov %r8, %rax; xchg %ax, %ax
//#ExpectInstructions:.text 31 mov %r16, %rax; xchg %ax, %ax
//#ExpectInstructions:.text 45 mov %r24, %rax; xchg %ax, %ax; ret

.text
.globl _start
_start:
    {rex2} lea tlsvar@TLSDESC(%rip), %rax
    call *tlsvar@TLSCALL(%rax)
    {rex2} lea tlsvar@TLSDESC(%rip), %r8
    mov %r8, %rax
    call *tlsvar@TLSCALL(%rax)
    lea tlsvar@TLSDESC(%rip), %r16
    mov %r16, %rax
    call *tlsvar@TLSCALL(%rax)
    lea tlsvar@TLSDESC(%rip), %r24
    mov %r24, %rax
    call *tlsvar@TLSCALL(%rax)
    ret

.section .note.GNU-stack,"",@progbits
