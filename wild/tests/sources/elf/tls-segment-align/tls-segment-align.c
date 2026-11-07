//#AbstractConfig:default
// LLD 23 removes the high-part TLSLE addition when the offset fits in 12 bits.
// TODO: Implement this relaxation in Wild.
//#DiffIgnore:rel.missing-opt.R_AARCH64_TLSLE_ADD_TPREL_HI12.ReplaceWithNop.*
//#LinkerDriver:gcc
//#Object:ptr_black_box.c
//#DiffIgnore:section.tdata.alignment
//#DiffIgnore:section.rodata
//#DiffIgnore:section.rela.dyn
//#DiffIgnore:section.data
//#DiffIgnore:section.sdata
//#DiffIgnore:rel.extra-got-plt-got

//#Config:dynamic:default
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:static:default
// LLD uses lea for defined symbols and leaves undefined-weak GOT loads unrelaxed.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.static-non-pie
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-non-pie
// Alpine's compiler driver produces PIE even with -static. TODO: pass -no-pie automatically
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-pie
//#LinkArgs:-static
//#SkipArch:loongarch64
//#CompArgs:-fPIE

//#Config:compress-debug:default
//#CompArgs:-g
//#LinkArgs:-Wl,--compress-debug-sections=zlib
//#DiffIgnore:section.debug_*
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:compress-debug-static:static
//#SkipArch:loongarch64
//#CompArgs:-g
//#LinkArgs:-Wl,--compress-debug-sections=zlib
//#DiffIgnore:section.debug_*

//#Config:static-loongarch64:static
//#Arch:loongarch64
// LLD 21 produces crashing LoongArch IFUNC binaries, including static libc users.
// TODO: Remove the LoongArch BFD-only configurations once the oldest supported LLD is 22;
// these cases pass with LLD 22.
//#ReferenceLinkers:bfd

//#Config:compress-debug-static-loongarch64:compress-debug-static
//#Arch:loongarch64
// LLD 21 produces crashing LoongArch IFUNC binaries, including static libc users.
//#ReferenceLinkers:bfd

#include "../common/ptr_black_box.h"

__thread unsigned long tls_var_a = 0x1122334455667788UL;
__thread char tls_var_b[8] __attribute__((aligned(256)));

int main(void) {
  if (tls_var_a != 0x1122334455667788UL) {
    return 1;
  }
  if (tls_var_b[0] != 0) {
    return 2;
  }
  if (ptr_to_int(tls_var_b) % 256 != 0) {
    return 3;
  }
  return 42;
}
