// When BTI is enabled, a range-extension thunk branches indirectly (`br x16`),
// so its target must begin with a landing pad. If the target already starts
// with `bti c`, no extra landing pad should be inserted: the target itself sits
// where the pad would otherwise be, and the thunk branches straight to it.
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
//#ExpectInstructions:.text 0x8000018 mov w0, #7

.text
.p2align 2

.globl caller
.type caller, %function
caller:
    bl far_target
    ret
