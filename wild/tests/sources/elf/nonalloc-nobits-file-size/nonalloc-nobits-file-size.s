/*
There's not really any good use for a nonalloc NOBITS section, but if anyone accidentally creates
one, it shouldn't consume a bunch of zeros in the file. We only test against BFD because LLD does
take up space in the file in this case.

//#AbstractConfig:default
//#Arch:x86_64
//#RunEnabled:false
//#ReferenceLinkers:bfd
//#LinkArgs:--no-gc-sections
//#ExpectSection:.nonalloc_nobits flags='',type=0x8
//#ExpectSym:nobits_data section=".nonalloc_nobits",size=1048576
//#MaxFileSize:65536
//#DiffIgnore:section.debug_*

//#Config:none:default
//#LinkArgs:--compress-debug-sections=none

//#Config:zlib:default
//#LinkArgs:--compress-debug-sections=zlib
*/

.text
.globl _start
_start:
    ret

.section .debug_test,"",%progbits
    .zero 1024

.section .nonalloc_nobits,"",%nobits
.globl nobits_data
.type nobits_data,%object
nobits_data:
    .zero 1048576
.size nobits_data,.-nobits_data

.section .note.GNU-stack,"",%progbits
