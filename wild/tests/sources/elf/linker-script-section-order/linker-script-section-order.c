//#AbstractConfig:default
//#Object:runtime.c
//#SkipArch:riscv64
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:data_before_text:default
//#LinkerScript:data-before-text.ld
//#CompArgs:-DTEST_DATA_BEFORE_TEXT
//#ExpectSym:data_start address=0x500000
//#ExpectSym:text_start address=0x600000
//#ExpectSym:rodata_start address=0x700000
//#ExpectSym:bss_start address=0x800000
//#NoSection:.relro_padding

//#Config:custom_sections:default
//#LinkerScript:custom-sections.ld
//#CompArgs:-DTEST_CUSTOM_SECTIONS
//#ExpectSym:first_start address=0x400000
//#ExpectSym:text_start address=0x500000
//#ExpectSym:middle_start address=0x600000
//#ExpectSym:data_start address=0x700000
//#ExpectSym:last_start address=0x800000
//#NoSection:.relro_padding

//#Config:compressed:data_before_text
//#CompArgs:-g
//#LinkArgs:--compress-debug-sections=zlib
//#DiffIgnore:section.debug_*
//#ExpectSection:.debug_info flags=C

#include <stddef.h>

#include "../common/runtime.h"

#if defined(TEST_DATA_BEFORE_TEXT)

extern char data_start;
extern char text_start;
extern char rodata_start;
extern char bss_start;

int data_val = 11;
const int rodata_val = 22;
int bss_val;

void _start(void) {
  if ((size_t)&data_start != 0x500000) {
    exit_syscall(1);
  }
  if ((size_t)&text_start != 0x600000) {
    exit_syscall(2);
  }
  if ((size_t)&rodata_start != 0x700000) {
    exit_syscall(3);
  }
  if ((size_t)&bss_start != 0x800000) {
    exit_syscall(4);
  }

  if ((size_t)&data_start >= (size_t)&text_start) {
    exit_syscall(5);
  }
  if ((size_t)&text_start >= (size_t)&rodata_start) {
    exit_syscall(6);
  }
  if ((size_t)&rodata_start >= (size_t)&bss_start) {
    exit_syscall(7);
  }

  if (data_val != 11 || rodata_val != 22 || bss_val != 0) {
    exit_syscall(8);
  }

  data_val = 33;
  bss_val = 44;
  if (data_val != 33 || bss_val != 44) {
    exit_syscall(9);
  }

  exit_syscall(42);
}

#elif defined(TEST_CUSTOM_SECTIONS)

extern char first_start;
extern char text_start;
extern char middle_start;
extern char data_start;
extern char last_start;

__attribute__((section(".first"))) int first_val = 1;
__attribute__((section(".middle"))) int middle_val = 2;
int data_val = 3;
__attribute__((section(".last"))) int last_val = 4;

void _start(void) {
  if ((size_t)&first_start != 0x400000) {
    exit_syscall(1);
  }
  if ((size_t)&text_start != 0x500000) {
    exit_syscall(2);
  }
  if ((size_t)&middle_start != 0x600000) {
    exit_syscall(3);
  }
  if ((size_t)&data_start != 0x700000) {
    exit_syscall(4);
  }
  if ((size_t)&last_start != 0x800000) {
    exit_syscall(5);
  }

  if ((size_t)&first_start >= (size_t)&text_start) {
    exit_syscall(6);
  }
  if ((size_t)&text_start >= (size_t)&middle_start) {
    exit_syscall(7);
  }
  if ((size_t)&middle_start >= (size_t)&data_start) {
    exit_syscall(8);
  }
  if ((size_t)&data_start >= (size_t)&last_start) {
    exit_syscall(9);
  }

  if (first_val != 1 || middle_val != 2 || data_val != 3 || last_val != 4) {
    exit_syscall(10);
  }

  exit_syscall(42);
}

#endif
