/*
//#AbstractConfig:default
//#LinkerDriver:gcc
//#Shared:dynsym-ifunc-relaxation-size-shared.s
//#RunEnabled:false
//#DiffEnabled:false

//#AbstractConfig:non-pie:default
//#LinkArgs:-nostdlib -no-pie -Wl,--no-as-needed,--export-dynamic,--relax,-z,now

//#AbstractConfig:pie:default
//#LinkArgs:-nostdlib -pie -Wl,--no-as-needed,--export-dynamic,--relax,-z,now

//#Config:non-pie-riscv64:non-pie
//#Arch:riscv64
//#CompArgs:-march=rv64gc
//#CompSoArgs:-march=rv64gc
//#ExpectDynSym:relaxed_ifunc size=6

//#Config:pie-riscv64:pie
//#Arch:riscv64
//#CompArgs:-march=rv64gc
//#CompSoArgs:-march=rv64gc
//#ExpectDynSym:relaxed_ifunc section=".text",size=6

//#Config:non-pie-riscv32:non-pie
//#Arch:riscv32
//#CompArgs:-march=rv32gc
//#CompSoArgs:-march=rv32gc
//#LinkArgs:-march=rv32gc -mabi=ilp32d
//#LinkSoArgs:-march=rv32gc -mabi=ilp32d
//#ExpectDynSym:relaxed_ifunc size=4

//#Config:pie-riscv32:pie
//#Arch:riscv32
//#CompArgs:-march=rv32gc
//#CompSoArgs:-march=rv32gc
//#LinkArgs:-march=rv32gc -mabi=ilp32d
//#LinkSoArgs:-march=rv32gc -mabi=ilp32d
//#ExpectDynSym:relaxed_ifunc section=".text",size=4
*/

.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
    call relaxed_ifunc
    ret
.size _start, .-_start

.globl relaxed_ifunc
.type relaxed_ifunc, @gnu_indirect_function
relaxed_ifunc:
    call nearby_func
    ret
.size relaxed_ifunc, .-relaxed_ifunc

.type nearby_func, @function
nearby_func:
    ret
.size nearby_func, .-nearby_func
