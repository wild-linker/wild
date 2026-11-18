/*
//#Arch: riscv32
//#LinkArgs: -shared
//#RunEnabled: false
*/

.text
.globl test_tls_ie
.type test_tls_ie, @function
test_tls_ie:
    la.tls.ie a0, var_ie
    ret

.globl test_tls_gd
.type test_tls_gd, @function
test_tls_gd:
    la.tls.gd a0, ext_gd_var
    ret

.globl test_local_gd
.type test_local_gd, @function
test_local_gd:
    la.tls.gd a0, local_gd_var
    ret

.section .tbss, "awT", @nobits
.globl var_ie
.type var_ie, @object
.align 2
var_ie:
    .zero 4

.globl local_gd_var
.type local_gd_var, @object
.align 2
local_gd_var:
    .zero 4
