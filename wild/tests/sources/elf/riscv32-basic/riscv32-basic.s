/*
//#Arch: riscv32
//#LinkArgs: -no-pie
*/

.text
.globl _start

_start:
    li a0, 42        # Exit status code (42)
    li a7, 93        # Linux RV32 exit syscall number
    ecall            # Transfer control to the OS kernel
