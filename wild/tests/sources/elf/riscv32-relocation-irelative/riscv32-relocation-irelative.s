/*
//#AbstractConfig: default
//#Arch: riscv32
//#RunEnabled: false
//#DiffIgnore: .dynamic.DT_RELA*

//#Config: pie:default
//#LinkArgs: -pie

//#Config: no-pie:default
//#LinkArgs: -no-pie
*/

.text
.globl _start
.type _start, @function
_start:
    call my_ifunc
    ret

.globl my_ifunc
.type my_ifunc, @gnu_indirect_function
my_ifunc:
    lla a0, resolver_target
    ret

.globl resolver_target
.type resolver_target, @function
resolver_target:
    ret
