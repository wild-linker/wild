/*
//#AbstractConfig:default
//#Arch:x86_64,aarch64,loongarch64
//#RunEnabled:false
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--no-gc-sections -z noexecstack
//#ExpectSection:.nonalloc_nobits flags='',type=0x8
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

.section .debug_test,"",@progbits
    .zero 1024

.section .nonalloc_nobits,"",@nobits
    .zero 16
