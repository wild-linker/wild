//#AbstractConfig:base
//#Cross:false
//#Object:../common/runtime.c
//#ReferenceLinkers:bfd
//#RunEnabled:false

//#Config:invalid:base
//#Env:LDEMULATION=not-an-emulation
//#ExpectError:not-an-emulation
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:valid-x86_64:base
//#Arch:x86_64
//#Env:LDEMULATION=elf_x86_64
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-dir-output-name-x86_64:valid-x86_64
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:-o elf -z now
//#RunEnabled:true

//#AbstractConfig:command-line-override-x86_64-base:base
//#Arch:x86_64
//#Env:LDEMULATION=not-an-emulation

//#Config:command-line-override-x86_64:command-line-override-x86_64-base
//#LinkArgs:-m elf_x86_64 -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:repeated-command-line-override-x86_64:command-line-override-x86_64-base
//#LinkArgs:-melf_x86_64 -m elf_x86_64 -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:invalid-then-valid-x86_64:command-line-override-x86_64-base
//#LinkArgs:-m invalid-emulation -m elf_x86_64 -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:invalid-then-valid-attached-x86_64:command-line-override-x86_64-base
//#LinkArgs:-minvalid-emulation -melf_x86_64 -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:valid-then-invalid-x86_64:valid-x86_64
//#LinkArgs:-m elf_x86_64 -m invalid-emulation -z now
//#ExpectError:invalid-emulation

//#Config:invalid-then-invalid-x86_64:valid-x86_64
//#LinkArgs:-m first-invalid-emulation -m last-invalid-emulation -z now
//#ExpectError:last-invalid-emulation

//#Config:invalid-command-line-x86_64:valid-x86_64
//#LinkArgs:-m not-an-emulation -z now
//#ExpectError:not-an-emulation

//#Config:valid-aarch64:base
//#Arch:aarch64
//#Env:LDEMULATION=aarch64linux
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:command-line-override-aarch64:base
//#Arch:aarch64
//#Env:LDEMULATION=not-an-emulation
//#LinkArgs:-m aarch64linux -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:valid-riscv64:base
//#Arch:riscv64
//#Env:LDEMULATION=elf64lriscv
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:command-line-override-riscv64:base
//#Arch:riscv64
//#Env:LDEMULATION=not-an-emulation
//#LinkArgs:-m elf64lriscv -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:valid-loongarch64:base
//#Arch:loongarch64
//#Env:LDEMULATION=elf64loongarch
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:command-line-override-loongarch64:base
//#Arch:loongarch64
//#Env:LDEMULATION=not-an-emulation
//#LinkArgs:-m elf64loongarch -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:valid-ppc64le:base
//#Arch:ppc64le
//#Env:LDEMULATION=elf64lppc
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:command-line-override-ppc64le:base
//#Arch:ppc64le
//#Env:LDEMULATION=not-an-emulation
//#LinkArgs:-m elf64lppc -z now
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
