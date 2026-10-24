//#Arch:aarch64
//#Compiler:clang
//#Object:padding1.s
//#Object:padding2.s
//#Object:target.s
//#ReferenceLinkers:
//#RunEnabled:false
//#DiffEnabled:false
//#LinkArgs:-shared -z force-bti
//#ExpectWarning:.*: -z force-bti: file does not have .*BTI property
//#ExpectInstructions:.text 0x8000014 bti c

.text
.p2align 2

.globl caller
.type caller, %function
caller:
    bl far_target
    ret
