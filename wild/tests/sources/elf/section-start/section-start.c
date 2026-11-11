//#Config:section-start
// Wild and BFD use an absolute move where LLD uses a PC-relative lea.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.dynamic-non-pie
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now -Wl,--section-start=.foo=0x1000000 -no-pie
//#ExpectSym:foo address=0x1000000
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:section.rodata.alignment
//#DiffIgnore:section.data
//#DiffIgnore:section.sdata
//#SkipArch:loongarch64,ppc64le
//#CompArgs:-fPIE

/* BFD rejects the code on loongarch: relocation truncated to fit: R_LARCH_B26
   against symbol `foo' defined in .foo section. */

#include <stdio.h>

__attribute__((section(".foo"))) void foo() { printf("foo\n"); }

int main() {
  foo();

  return 42;
}
