// This script relies on GNU orphan placement and segment permissions for execution.
// TODO: Make the script's segment permissions and page boundaries explicit while preserving
// its address assertions, then restore LLD coverage.
//#ReferenceLinkers:bfd
//#LinkerScript:linker-script-section-start.ld
//#Object:runtime.c
//#DiffIgnore:segment.LOAD.RW.alignment
// RISC-V: BFD complains about missing __global_pointer$ (defined in the default
// linker script)
//#SkipArch:riscv64
//#ExpectSym:foo address=0x1000000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

__attribute__((used, section(".foo"))) int foo = 7;

void begin_here(void) { exit_syscall(42); }
