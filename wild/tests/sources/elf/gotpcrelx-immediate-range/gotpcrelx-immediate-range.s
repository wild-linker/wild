//#AbstractConfig:default
//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-Wa,-mrelax-relocations=yes
//#LinkArgs:-z noexecstack
//#ReferenceLinkers:bfd,lld

//#Config:mov:default
//#CompArgs:-Wa,--defsym,OP=0
//#LinkArgs:--defsym=target=0x80000000
//#LinkArgs:-no-pie

//#Config:sub:default
//#CompArgs:-Wa,--defsym,OP=1
//#LinkArgs:--defsym=target=0x80000000
//#LinkArgs:-no-pie

//#Config:cmp:default
//#CompArgs:-Wa,--defsym,OP=2
//#LinkArgs:--defsym=target=0x80000000
//#LinkArgs:-no-pie

//#Config:negative:default
// LLD keeps the GOT load; BFD and Wild use a signed immediate.
//#DiffIgnore:rel.R_X86_64_32S.R_X86_64_32S
//#CompArgs:-Wa,--defsym,OP=0
//#LinkArgs:--defsym=target=0xffffffffffffffff
//#LinkArgs:-no-pie

//#Config:wide:default
//#CompArgs:-Wa,--defsym,OP=0
//#LinkArgs:--defsym=target=0x100000000
//#LinkArgs:-no-pie

//#Config:signed-max:default
//#ReferenceLinkers:bfd
//#CompArgs:-Wa,--defsym,OP=0
//#LinkArgs:--defsym=target=0x7fffffff
//#ExpectInstructions:.text 10 mov $0x7fffffff, %rax
//#LinkArgs:-no-pie

//#Config:signed-min:default
//#ReferenceLinkers:bfd
//#CompArgs:-Wa,--defsym,OP=0
//#LinkArgs:--defsym=target=0xffffffff80000000
//#ExpectInstructions:.text 10 mov $-0x80000000, %rax
//#LinkArgs:-no-pie

//#Config:below-signed-min:default
//#CompArgs:-Wa,--defsym,OP=0
//#LinkArgs:--defsym=target=0xffffffff7fffffff
//#LinkArgs:-no-pie

//#Config:high-address:default
// LLD uses a PC-relative LEA; BFD and Wild retain the GOT load.
//#DiffIgnore:rel.R_X86_64_REX_GOTPCRELX.R_X86_64_REX_GOTPCRELX
//#CompArgs:-Wa,--defsym,OP=0,--defsym,DEFINED_TARGET=1
//#LinkArgs:-Ttext=0x80000000
//#LinkArgs:-no-pie

//#Config:mov32-wide:default
//#CompArgs:-Wa,--defsym,OP=3
//#LinkArgs:--defsym=target=0x100000001
//#LinkArgs:-no-pie

//#Config:mov32-max:default
//#ReferenceLinkers:bfd
//#CompArgs:-Wa,--defsym,OP=3
//#LinkArgs:--defsym=target=0xffffffff
//#ExpectSectionBytes:.text=0xc7c0ffffffff 10..16
//#LinkArgs:-no-pie

.text
.globl _start
_start:
    movabs $target, %rdx
.if OP == 0
    // The immediate in a 64-bit MOV is sign-extended from 32 bits.
    mov target@GOTPCREL(%rip), %rax
    cmp %rdx, %rax
.elseif OP == 1
    mov %rdx, %rax
    sub target@GOTPCREL(%rip), %rax
    test %rax, %rax
.elseif OP == 2
    cmp target@GOTPCREL(%rip), %rdx
.else
    mov target@GOTPCREL(%rip), %eax
    cmp %edx, %eax
.endif
    jne failed

    mov $42, %edi
    jmp exit
failed:
    mov $43, %edi
exit:
    mov $60, %eax
    syscall

.hidden target
.ifdef DEFINED_TARGET
.data
target:
    .quad 42
.endif
.section .note.GNU-stack,"",@progbits
