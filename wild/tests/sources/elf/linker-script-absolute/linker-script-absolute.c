//#ReferenceLinkers:bfd
//#Object:runtime.c
//#LinkerScript:linker-script-absolute.ld
//#SkipArch:riscv64
//#ExpectSym:abs_global is_abs=true,address=0x1000
//#ExpectSym:abs_global_calc is_abs=true,address=0x1500
//#ExpectSym:text_start section=".text",address=0x500000
//#ExpectSym:a section=".text",address=0x500020
//#ExpectSym:b section=".text",address=0x500010
//#ExpectSym:abs_const is_abs=true,address=0x500
//#ExpectSym:abs_dot is_abs=true,address=0x500000
//#ExpectSym:abs_sym_a is_abs=true,address=0x500020
//#ExpectSym:abs_sum_two is_abs=true,address=0xa00030
//#ExpectSym:abs_max_func is_abs=true,address=0x500020
//#ExpectSym:abs_nested is_abs=true,address=0x500020
//#ExpectSym:absolute_aligned is_abs=true,address=0x500000
//#ExpectSym:aligned_absolute is_abs=true,address=16
//#ExpectSym:absolute_plus is_abs=true,address=24
//#ExpectSym:absolute_plus_comparison is_abs=true,address=17
//#ExpectSym:sum_absolute section=".text",address=0x500018
//#ExpectSym:sum_absolute_addresses section=".text",address=0xf00000
//#ExpectSym:absolute_ternary is_abs=true,address=24
//#ExpectSym:absolute_max is_abs=true,address=24
//#ExpectSym:abs_max_mixed is_abs=true,address=0x500000
//#ExpectSym:abs_max_mixed_rev is_abs=true,address=0x500000
//#ExpectSym:ternary_taken_rel section=".text",address=0x500000
//#ExpectSym:ternary_taken_abs is_abs=true,address=32
//#ExpectSym:ternary_num_vs_abs section=".text",address=0x500018
//#ExpectSym:ternary_abs_vs_num is_abs=true,address=24
//#ExpectSym:abs_land section=".text",address=0x500001
//#ExpectSym:abs_lor section=".text",address=0x500001
//#ExpectSym:abs_lnot is_abs=true,address=0
//#ExpectSym:padding is_abs=true,address=16
//#ExpectSym:after_padding section=".text",address=0x500010
//#ExpectSym:absolute_last is_abs=true,address=0x500010
//#ExpectSym:absolute_first is_abs=true,address=0x500010
//#ExpectSym:data_start section=".data",address=0x600000
//#ExpectSym:e section=".data",address=0x600030

#include "../common/runtime.h"

int data_var = 42;

void _start(void) {
  runtime_init();
  exit_syscall(data_var);
}
