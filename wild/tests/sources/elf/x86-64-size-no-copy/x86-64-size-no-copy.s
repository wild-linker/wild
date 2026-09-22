//#Arch:x86_64
//#Mode:dynamic
//#ReferenceLinkers:bfd,lld
//#DiffMatchAny:true
//#Shared:x86-64-size-no-copy-def.s
//#RunEnabled:false
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_FLAGS_1.NOW

.globl _start
.text
_start:
    ret

.data
.long foo@SIZE
