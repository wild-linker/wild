//#LinkArgs:--entry _start
//#RunEnabled:false
//#DiffIgnore:section.got
//#ExpectSym:abs_sym is_abs,address=0xCAFECAFE

.text
.globl _start
_start:
call .text
    ret

.global abs_sym
.set abs_sym, 0xCAFECAFE
