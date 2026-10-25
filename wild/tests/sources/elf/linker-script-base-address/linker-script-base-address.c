//#AbstractConfig:default
//#Object:runtime.c
// RISC-V: BFD complains about missing __global_pointer$ (defined in the default linker script)
//#SkipArch:riscv64
//#ReferenceLinkers:bfd,lld

//#Config:page_aligned_x86_64:default
//#Arch:x86_64
//#LinkerScript:page-aligned.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x5ff000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x800000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:phdrs_no_base_address:default
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld
//#LinkerScript:phdrs-no-base-address.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x3ff000
//#ExpectSection:.text address=0x400000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:nonalloc_first:default
//#CompArgs:-g
//#LinkArgs: --no-gc-sections
//#LinkerScript:nonalloc-first.ld
//#ExpectSection:.debug_info address=0,index=1
//#ExpectSection:.debug_str address=0,index=2
//#ExpectSection:.debug_line address=0
//#ExpectSection:.text address=0x400000

//#Config:nonalloc_first_x86:nonalloc_first
//#Arch:x86_64
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x3ff000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:nonalloc_first_aarch64:nonalloc_first
//#Arch:aarch64
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x3f0000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:nonalloc_first_compressed:nonalloc_first_x86
//#LinkArgs:--compress-debug-sections=zlib

//#Config:nonalloc_secondary:default
//#Arch:x86_64
//#Object:nonalloc-secondary.s
//#LinkerScript:nonalloc-secondary.ld
//#ExpectSection:.metadata address=0
//#ExpectSectionBytes:.metadata=0x0102
//#ExpectSection:.text address=0x400000
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x3ff000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:page_aligned_aarch64:default
//#Arch:aarch64
//#LinkerScript:page-aligned.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x5f0000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x800000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:sizeof_headers:default
//#LinkerScript:sizeof-headers.ld
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*],vaddr=0x400000
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*],vaddr=0x800000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-hdrs:default
//#LinkerScript:no-hdrs.ld
//#ReferenceLinkers:bfd
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:base_too_small:default
//#LinkerScript:base-too-small.ld
//#ReferenceLinkers:
//#ExpectErrorWild:(?i-u)base address \(0x10\) is less than the size of headers
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:phdrs_base_too_small:default
//#LinkerScript:phdrs-base-too-small.ld
//#ReferenceLinkers:
//#ExpectErrorWild:(?i-u)base address \(0x10\) is less than the size of headers
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:phdrs_base_too_small_nohdrs:default
//#LinkerScript:phdrs-base-too-small-nohdrs.ld
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#ExpectSym:_start address=0x10
//#ExpectProgramHeader:LOAD flags=RWX,sections=[.text,.data,*],vaddr=0x10
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-base-address:default
//#ReferenceLinkers:bfd
//#LinkerScript:no-base-address.ld
//#ExpectSym:_start address=0x600000
//#ExpectSym:other address=0x700000
//#ExpectSym:rodata address=0x500000
//#DiffIgnore:segment.LOAD.RX.alignment
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:below_default_base_x86:default
//#Arch:x86_64
//#LinkerScript:below-default-base.ld
//#ExpectSection:.text address=0x3fe800
//#ExpectProgramHeader:LOAD vaddr=0x3fe000,sections=[.text,*]
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:below_default_base_aarch64:default
//#Arch:aarch64
//#LinkerScript:below-default-base.ld
//#ExpectSection:.text address=0x3fe800
//#ExpectProgramHeader:LOAD vaddr=0x3f0000,sections=[.text,*]
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:zero_base:default
//#CompArgs: -fPIC
//#LinkerScript:zero-base.ld
//#ExpectSection:.text address=0x1000
//#ExpectProgramHeader:LOAD vaddr=0,sections=[.text,*]
//#RunEnabled:false

//#Config:pie:zero_base
//#Mode:dynamic
//#LinkArgs:-pie
//#DiffMatchAny:true

//#Config:shared:zero_base
//#LinkArgs:-shared

#include "../common/runtime.h"

int value = 42;

void _start(void) {
  runtime_init();
  exit_syscall(value);
}

__attribute__((section(".text.first"), aligned(1))) const char symbol1 = 1;

__attribute__((section(".text.second"), aligned(1))) const char symbol2 = 2;

__attribute__((section(".rodata.first"), aligned(1))) const char rodata = 1;

__attribute__((section(".other"), aligned(1), used)) void other() { return; };
