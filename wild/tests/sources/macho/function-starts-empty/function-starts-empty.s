//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
// LLD includes the assembler's temporary label for the empty __text section as a function.
//#ReferenceLinkers:ld
//#RunEnabled:false
//#ExpectFunctionStarts:
//#ExpectLoadCommandSize:LC_FUNCTION_STARTS 16

.data
.globl _data
_data:
    .long 42
