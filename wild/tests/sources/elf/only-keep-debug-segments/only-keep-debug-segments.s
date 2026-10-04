/*
//#Arch:x86_64,aarch64,loongarch64
//#LinkerScript:layout.ld
//#LinkArgs:--only-keep-debug --no-gc-sections -z norelro
//#ReferenceLinkers:
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectProgramHeader:LOAD sections=[*],flags=RX,file-size=768
//#ExpectProgramHeader:NOTE sections=[*],flags=R,file-size=480,mem-size=512,offset=288
//#ExpectProgramHeader:TLS sections=[*],flags=R,file-size=0,mem-size=32,offset=768
//#ExpectProgramHeader:LOAD sections=[*],flags=RW,file-size=0,offset=768
//#ExpectSection:.text type=8
//#ExpectSection:.tdata type=8
//#ExpectSection:.data type=8
*/

.section .note.first,"a",@note
.balign 64
    .long 4, 0, 1
    .asciz "TST"

.section .note.second,"a",@note
.balign 256
    .long 4, 0, 2
    .asciz "TST"

.text
.balign 512
.globl _start
_start:
    .zero 16

.section .tdata,"awT",@progbits
    .zero 16

.section .tbss,"awT",@nobits
    .zero 16

.data
    .zero 16
