//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-Wa,-mrelax-relocations=no
//#LinkArgs:-z noexecstack
//#ReferenceLinkers:bfd,lld

//#Config:mov32:default
//#CompArgs:-Wa,--defsym,FORM=0
//#LinkArgs:-no-pie

//#Config:mov64:default
//#CompArgs:-Wa,--defsym,FORM=1
//#LinkArgs:-no-pie

//#Config:apx:default
//#CompArgs:-Wa,--defsym,FORM=2
//#RunEnabled:false
//#ExpectSectionBytes:.text=0xd5488b05 0..4
//#LinkArgs:-no-pie

.text
.globl _start
_start:
.if FORM == 0
    // Read the upper half of the GOT entry.
    mov 0(%rip), %eax
    .reloc .-4, R_X86_64_GOTPCRELX, target
    mov target@GOTPCREL+4(%rip), %edx
.elseif FORM == 1
    mov 0(%rip), %rax
    .reloc .-4, R_X86_64_REX_GOTPCRELX, target-5
    mov target@GOTPCREL-1(%rip), %rdx
.else
    mov 0(%rip), %r16
    .reloc .-4, R_X86_64_CODE_4_GOTPCRELX, target-5
    mov %r16, %rax
    mov target@GOTPCREL-1(%rip), %rdx
.endif
    cmp %rdx, %rax
    jne failed
    mov $42, %edi
    jmp exit
failed:
    mov $43, %edi
exit:
    mov $60, %eax
    syscall

.globl target
.hidden target
.set target, 0x12345678
.section .note.GNU-stack,"",@progbits
