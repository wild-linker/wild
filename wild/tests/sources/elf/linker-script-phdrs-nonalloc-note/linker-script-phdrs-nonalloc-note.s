//#Arch:x86_64
//#Mode:static
//#RunEnabled:false
//#LinkerScript:linker-script-phdrs-nonalloc-note.ld
//#ExpectSection:.debug_test flags=''
//#ExpectSection:.debug_test2 flags=''
//#ExpectSection:.debug_test3 flags=''
//#ExpectSection:.debug_test4 flags=''
//#ExpectProgramHeader:NOTE sections=[.note.one,.note.two],vaddr=0x400004,file-size=0x28,mem-size=0x28
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data.one,.data.two,*]
//#ExpectProgramHeader:TLS sections=[.tdata.one,.tdata.two]
//#ExpectProgramHeader:NULL sections=[.null.one,.null.two]

//#Config:only-keep-debug:default
//#LinkArgs:--only-keep-debug
//#ReferenceLinkers:

.text
.globl _start
_start:
    ret

.section .note.one,"a",@note
.balign 4
    .long 4, 4, 1
    .asciz "TST"
    .long 0

.section .note.two,"a",@note
.balign 4
    .long 4, 4, 2
    .asciz "TST"
    .long 0

.section .debug_test,"",@progbits
    .asciz "a non-allocated section"

.section .data.one,"aw",@progbits
    .long 0x11111111

.section .debug_test2,"",@progbits
    .asciz "second non-allocated section"

.section .data.two,"aw",@progbits
    .long 0x22222222

.section .tdata.one,"awT",@progbits
    .long 0x33333333

.section .debug_test3,"",@progbits
    .asciz "third non-allocated section"

.section .tdata.two,"awT",@progbits
    .long 0x44444444

.section .null.one,"a",@progbits
    .long 0x55555555

.section .debug_test4,"",@progbits
    .asciz "fourth non-allocated section"

.section .null.two,"a",@progbits
    .long 0x66666666
