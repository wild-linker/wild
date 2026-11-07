//#AbstractConfig:default
//#LinkerDriver:clang
//#ExpectFunctionStarts:_main _alias _local_function _other_function
//#ExpectLoadCommandSize:LC_FUNCTION_STARTS 16

//#Config:executable:default

//#Config:dylib:default
//#LinkArgs:-dynamiclib
//#RunDynSym:main

//#Config:enabled:default
//#LinkArgs:-Wl,-function_starts

//#Config:reenabled:default
//#LinkArgs:-Wl,-no_function_starts,-function_starts

//#Config:disabled
//#LinkerDriver:clang
//#LinkArgs:-Wl,-function_starts,-no_function_starts
//#ExpectFunctionStarts:none

//#Config:disabled-dylib
//#LinkerDriver:clang
//#LinkArgs:-dynamiclib -Wl,-no_function_starts
//#RunDynSym:main
//#ExpectFunctionStarts:none

.text
.p2align 2
.globl _main, _alias
_main:
_alias:
    mov w0, #42
    ret

// Force a multi-byte delta and retain a local function with no references or unwind entry.
.space 256
_local_function:
    ret

.section __TEXT,__other,regular,pure_instructions
.p2align 2
.globl _other_function
_other_function:
    // The imported function's linker-generated stub must not appear in the table.
    b _getpid

// Data in __TEXT must not be mistaken for code.
.section __TEXT,__const
.globl _constant
_constant:
    .long 42

.data
.globl _data
_data:
    .long 42

.globl _absolute
_absolute = 0x1234
