//#Config:default
//#Mode:static
//#CompArgs:-fno-pie
//#LinkArgs:-no-pie
//#Object:runtime.c
//#LinkerScript:linker-script-empty-section.ld
//#ReferenceLinkers:bfd,lld
//#ExpectSym:empty_0_start address=0x4000b0
//#ExpectSym:empty_1_start address=0x408000
//#ExpectSym:empty_2_start address=0x418000
//#NoSection:.empty_7
//#SkipArch:riscv64

//#Config:flags:default
//#ReferenceLinkers:lld
//#ExpectSection:.empty_0 type=1,flags=A
//#ExpectSection:.empty_1 type=1,flags=A
//#ExpectSection:.empty_2 type=1,flags=WA
//#ExpectSection:.empty_3 type=1,flags=W
//#ExpectSym:empty_3_start address=0x0
//#ExpectSection:.empty_4 type=8,flags=WA
//#SkipArch:riscv64,x86_64

//#Config:x86:flags
//#Arch:x86_64
//#CompArgs:-DX86
//#ExpectSection:.empty_5 type=1,flags=''
//#ExpectSection:.empty_6 type=1,flags=W

#include "../common/runtime.h"

int exit_code = 42;

void _start() {
  runtime_init();
  exit_syscall(exit_code);
}

#ifdef X86
__attribute__((section(".exec_only_sec,\"x\",@progbits#"))) int exec_only_var = 0;
__attribute__((section(".write_only_sec,\"w\",@progbits#"))) int write_only_var = 0;
#endif
