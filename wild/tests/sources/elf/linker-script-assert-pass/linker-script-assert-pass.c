//#LinkArgs:-T ./linker-script-assert-pass.ld
//#RunEnabled:false
//#DiffEnabled:false
//#Object:runtime.c
// RISC-V: BFD complains about missing __global_pointer$ (defined in the default linker script)
//#SkipArch:riscv64
//#ExpectSym:symbol10 section=".data",offset-in-section=0x10040
//#ExpectSym:symbol11 address=0x80
//#ExpectSym:symbol10_assert section=".data",offset-in-section=1
//#ExpectSym:symbol11_assert address=0x1
//#ExpectSym:symbol13 section=".text"
//#ExpectSym:symbol13a section=".text"
//#ExpectSym:symbol14 is_abs=true,address=0x1000
//#ExpectSym:symbol14a is_abs=true,address=0x1000
//#ExpectSym:min_same_sec section=".text"
//#ExpectSym:max_same_sec section=".text"
//#ExpectSym:min_diff_sec is_abs=true
//#ExpectSym:max_diff_sec is_abs=true
//#ExpectSym:min_with_abs is_abs=true,address=0x10
//#ExpectSym:max_with_abs is_abs=true
//#ExpectSym:symbol13_1 section=".text"
//#ExpectSym:symbol13_2 section=".text"
//#ExpectSym:symbol13_3 section=".text"
//#ExpectSym:symbol13_text_offset section=".text"
//#ExpectSym:diff_same_sec is_abs=true,address=0x10
//#ExpectSym:sym_log_and_true is_abs=true,address=0x1
//#ExpectSym:sym_log_not is_abs=true,address=0x0
//#ExpectSym:sym_bit_and is_abs=true,address=0x1000
//#ExpectSym:sym_neg is_abs=true
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int symbol4 = 0x4000;
int symbol6 __attribute__((weak));
int symbol7 __attribute__((weak)) = 0x7000;
extern int symbol8 __attribute__((weak));

void _start() { exit_syscall(symbol4 + symbol6 + symbol7 + symbol8); }
