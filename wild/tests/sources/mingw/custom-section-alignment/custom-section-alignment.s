//#LinkArgs:--entry _start
//#ExpectSection:.text address=0x140001000
//#ExpectSection:.rdata address=0x140002000
//#ExpectSection:.custom1 address=0x140003000
//#ExpectSection:.custom2 address=0x140004000
//#ExpectSection:.custom3 address=0x140005000
//#RunEnabled:false
//#ReferenceLinkers:

.text
.globl _start
_start:
call .text
    call .custom1
    call .custom2
    call .custom2$a
    call .custom3
    ret

// .CRT sections are currently handled as secondary sections, make sure they
//  don't introduce spurious offsets.
.section .CRT$A,"a"
    ret
.section .CRT$B,"a"
    ret

.section .custom1,"a"
    ret
.section .custom2,"a"
    ret
.section .custom2$a,"a"
    ret
.section .custom3,"a"
    ret
