//#AbstractConfig:base
//#Arch:x86_64
//#LinkArgs:-shared -z now
//#ReferenceLinkers:lld,bfd

//#Config:default:base
//#RunEnabled:false

//#Config:with-tls:base
//#CompArgs:-Wa,--defsym,WITH_TLS=1
//#RunDynSym:check_descriptor
// BFD subtracts the TLS segment address from the undefined symbol's zero value.
// LLD uses a zero offset, which is the behavior we match.
//#ReferenceLinkers:lld
//#ExpectProgramHeader:TLS file-size=8,mem-size=8,sections=[.tdata]

//#Config:default-visibility:with-tls
//#CompArgs:-Wa,--defsym,DEFAULT_VISIBILITY=1
//#ReferenceLinkers:lld,bfd
//#ExpectDynSym:missing binding=weak,type=tls

.text
.globl check_descriptor
.type check_descriptor,@function
check_descriptor:
    lea missing@tlsgd(%rip), %rdi
    xor %eax, %eax
    cmpq $0, 8(%rdi)
    jne .Lfail
    mov $42, %eax
.Lfail:
    ret

.weak missing
.ifndef DEFAULT_VISIBILITY
.hidden missing
.endif
.type missing,@tls_object

.ifdef WITH_TLS
.section .tdata,"awT",@progbits
.globl defined_tls
.type defined_tls,@tls_object
defined_tls:
    .quad 42
.endif

.section .note.GNU-stack,"",@progbits
