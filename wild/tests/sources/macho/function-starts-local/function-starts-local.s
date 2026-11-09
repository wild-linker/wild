//#AbstractConfig:default
//#LinkerDriver:clang
//#ExpectFunctionStarts:_main-8 _main
//#NoSym:ltmp0

//#Config:executable:default

//#Config:dylib:default
//#LinkArgs:-dynamiclib
//#RunDynSym:main

// The assembler drops Lhelper, leaving only ltmp0 at the start of __text.
// The helper must still have a function start even though ltmp0 is stripped
// from the output symbol table. There is no unwind entry to identify it.
.text
.p2align 2
Lhelper:
    mov w0, #42
    ret

.globl _main
_main:
    b Lhelper
