//#Arch:x86_64
//#LinkArgs:-static --emit-relocs --no-gc-sections
// BFD reverses .ctors into .init_array; LLD retains .ctors.
//#ReferenceLinkers:bfd
//#RunEnabled:false
//#DiffEnabled:false
//#ExpectSection:.init_array size=16
//#ExpectReloc:.init_array offset=0,target=bar,type=1,addend=0
//#ExpectReloc:.init_array offset=8,target=foo,type=1,addend=0

.globl _start, foo, bar
_start:
    ret
foo:
    ret
bar:
    ret

.section .ctors,"aw",@progbits
    .quad foo
    .quad bar

.section .note.GNU-stack,"",@progbits
