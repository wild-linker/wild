//#Arch:x86_64
//#Compiler:gcc
//#CompSoArgs:-Wa,-mrelax-relocations=yes
//#Mode:dynamic
//#LinkArgs:-E -z noexecstack
//#Shared:gotpcrelx-absolute-interposable-shared.s
//#ReferenceLinkers:bfd,lld
//#ExpectDynSym:target
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT

.globl target
.set target, 42

.text
.globl _start
_start:
    call load32
    cmp $42, %rax
    jne failed
    call load64
    cmp $42, %rax
    jne failed
    call subtract
    test %rax, %rax
    jne failed
    call compare
    cmp $1, %eax
    jne failed
    mov $42, %edi
    jmp exit
failed:
    mov $43, %edi
exit:
    mov $60, %eax
    syscall

.section .note.GNU-stack,"",@progbits
