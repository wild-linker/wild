//#AbstractConfig:base
//#Arch:x86_64
//#Compiler:gcc
//#Mode:dynamic
//#Shared:shared.s
//#LinkSoArgs:-no-pie
//#ReferenceLinkers:bfd,lld
//#DiffEnabled:false

//#Config:data:base
// musl cannot relocate read-only pointers in a PIE loaded by the kernel.
//#RequiresGlibc:true
//#LinkArgs:-pie -z notext --disable-new-dtags
//#ExpectDynamic:DT_TEXTREL

//#Config:function:data
//#CompArgs:-Wa,--defsym,FUNCTION=1

//#Config:data-relr:data
//#LinkArgs:-z pack-relative-relocs

//#Config:function-relr:function
//#LinkArgs:-z pack-relative-relocs

//#Config:eh-frame:function
//#CompArgs:-Wa,--defsym,EH_FRAME=1

//#Config:eh-frame-relr:eh-frame
//#LinkArgs:-z pack-relative-relocs

//#Config:data-text:data
//#RequiresGlibc:false
// LLD does not diagnose the missing base relocation for these read-only PIE pointers.
//#ReferenceLinkers:bfd
//#LinkArgs:-z text
//#ExpectError:read-only.*(relocation|symbol)|relocation.*read-only

//#Config:function-text:function
//#RequiresGlibc:false
// LLD does not diagnose the missing base relocation for these read-only PIE pointers.
//#ReferenceLinkers:bfd
//#LinkArgs:-z text
//#ExpectError:read-only.*(relocation|symbol)|relocation.*read-only

//#Config:data-warn:data
//#LinkArgs:--warn-textrel
//#ReferenceLinkers:bfd
//#ExpectWarningWild:output contains dynamic relocations in read-only sections

//#Config:function-warn:function
//#LinkArgs:--warn-textrel
//#ReferenceLinkers:bfd
//#ExpectWarningWild:output contains dynamic relocations in read-only sections

//#Config:data-no-pie:base
//#LinkArgs:-no-pie

//#Config:function-no-pie:data-no-pie
//#CompArgs:-Wa,--defsym,FUNCTION=1

.text
.globl _start
_start:
.ifdef EH_FRAME
    .cfi_startproc
    .cfi_personality 0, function
.endif
.ifdef FUNCTION
    call *pointer(%rip)
    mov %eax, %edi
.else
    mov pointer(%rip), %rax
    mov (%rax), %edi
.endif
    mov $60, %eax
    syscall
.ifdef EH_FRAME
    .cfi_endproc
.endif

.section .rodata,"a",@progbits
.balign 8
pointer:
.ifdef FUNCTION
    .quad function
.else
    .quad value
.endif

.section .note.GNU-stack,"",@progbits
