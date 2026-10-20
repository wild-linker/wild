//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-Wa,-mrelax-relocations=no
//#LinkArgs:-pie --no-dynamic-linker -z noexecstack
//#Config:weak:default
//#CompArgs:-Wa,--defsym,expected=0
//#ReferenceLinkers:bfd,lld
//#Config:absolute:default
//#CompArgs:-Wa,--defsym,expected=42
//#LinkArgs:--defsym target=42
// BFD incorrectly relaxes this load to LEA, adding the PIE load bias to 42.
//#ReferenceLinkers:lld
//#DiffIgnore:section.got.plt.entsize

//#Config:fixed:default
//#CompArgs:-Wa,--defsym,expected=42
//#LinkArgs:-no-pie --defsym target=42
//#ReferenceLinkers:bfd
//#NoSection:.got

.text
.globl _start
.weak target
_start:
    mov target@GOTPCREL(%rip), %rax
    cmp $expected, %rax
    jne failed
    mov $42, %edi
    jmp exit
failed:
    mov $43, %edi
exit:
    mov $60, %eax
    syscall

.section .note.GNU-stack,"",@progbits
