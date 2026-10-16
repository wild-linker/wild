//#Arch:x86_64
//#Compiler:gcc
//#Mode:dynamic
//#LinkArgs:-z noexecstack
//#Shared:tlsdesc-apx-initial-exec-shared.s
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#DiffIgnore:.dynamic.DT_NEEDED
// Check mov disp32(%rip), %r16 without the linker-dependent displacement.
//#ExpectSectionBytes:.text=0xd5488b05 0..4
//#ExpectInstructions:.text 8 mov %r16, %rax; xchg %ax, %ax; ret

.text
.globl _start
_start:
    // Removing the resolver call requires loading the GOT's TLS offset.
    lea tlsvar@TLSDESC(%rip), %r16
    mov %r16, %rax
    call *tlsvar@TLSCALL(%rax)
    ret

.section .note.GNU-stack,"",@progbits
