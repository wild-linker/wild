//#AbstractConfig:default
//#Object:runtime.c
// RISC-V: BFD complains about missing __global_pointer$ (defined in the default linker script)
//#SkipArch:riscv64

//#Config:page_aligned_x86_64:default
//#Arch:x86_64
//#LinkerScript:page-aligned.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x5ff000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x800000

//#Config:page_aligned_aarch64:default
//#Arch:aarch64
//#LinkerScript:page-aligned.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x5f0000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x800000

//#Config:sizeof_headers:default
//#LinkerScript:sizeof-headers.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x400000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x800000

//#Config:no-hdrs:default
//#LinkerScript:no-hdrs.ld
//#ReferenceLinkers:bfd

//#Config:base_too_small:default
//#LinkerScript:base-too-small.ld
//#ReferenceLinkers:
//#ExpectErrorWild:(?i-u)base address \(0x10\) is less than the size of headers

//#Config:phdrs_base_too_small:default
//#LinkerScript:phdrs-base-too-small.ld
//#ReferenceLinkers:
//#ExpectErrorWild:(?i-u)base address \(0x10\) is less than the size of headers

#include "../common/runtime.h"

int value = 42;

void _start(void) {
  runtime_init();
  exit_syscall(value);
}

__attribute__((section(".text.first"), aligned(1))) const char symbol1 = 1;

__attribute__((section(".text.second"), aligned(1))) const char symbol2 = 2;
