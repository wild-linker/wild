//#AbstractConfig:base
//#Object:../common/runtime.c
//#SkipArch:loongarch64

/* loongarch64: relocation truncated to fit at these address ranges */

//#Config:ttext:base
//#SkipArch:loongarch64,ppc64le
//#LinkArgs:-Ttext=0x700000 -no-pie
//#ExpectSym:_start address=0x700000
//#CompArgs:-fPIE

//#Config:tdata:base
//#SkipArch:loongarch64,ppc64le
//#LinkArgs:-Tdata=0x800000 -no-pie --no-gc-sections
//#ExpectSym:data_var address=0x800000
//#CompArgs:-fPIE

//#Config:tbss:base
//#SkipArch:loongarch64,ppc64le
//#LinkArgs:-Tbss=0x900000 -no-pie --no-gc-sections
//#ExpectSym:bss_var address=0x900000
//#CompArgs:-fPIE

//#Config:ttext-ppc64le:ttext
//#Arch:ppc64le
// These addresses are below LLD's default PPC64 image base.
// TODO: Use addresses above the PPC64 image base in all three -T* cases and update their symbol
// assertions, then restore LLD coverage.
//#ReferenceLinkers:bfd

//#Config:tdata-ppc64le:tdata
//#Arch:ppc64le
// These addresses are below LLD's default PPC64 image base.
//#ReferenceLinkers:bfd

//#Config:tbss-ppc64le:tbss
//#Arch:ppc64le
// These addresses are below LLD's default PPC64 image base.
//#ReferenceLinkers:bfd

#include "../common/runtime.h"

/* Verify -Tdata places .data at the given address */
__attribute__((used)) int data_var = 1;

/* Verify -Tbss places .bss at the given address */
__attribute__((used)) int bss_var;

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
