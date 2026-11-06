//#AbstractConfig:base
//#Arch:x86_64
//#LinkArgs:-shared -z now
//#ReferenceLinkers:lld,bfd

//#Config:default:base
//#RunEnabled:false

//#Config:with-tls:base
//#Object:tls-data.s
//#RunDynSym:check_offset
// BFD subtracts the TLS segment address from the undefined symbol's zero value.
// LLD uses a zero addend, which is the behavior we match.
//#ReferenceLinkers:lld
//#ExpectProgramHeader:TLS file-size=8,mem-size=8,sections=[.tdata]

.text
.globl get_offset
get_offset:
    mov missing@gottpoff(%rip), %rax
    ret

.weak missing
.hidden missing
.type missing,@tls_object

.section .note.GNU-stack,"",@progbits
