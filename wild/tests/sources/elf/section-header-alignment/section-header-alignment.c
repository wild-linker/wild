//#Arch:x86_64
//#ReferenceLinkers:bfd,lld
//#Object:runtime.c

#include <elf.h>

#include "../common/runtime.h"

extern const Elf64_Ehdr __ehdr_start;

void _start(void) {
  runtime_init();

  if (__ehdr_start.e_shoff % _Alignof(Elf64_Shdr) != 0) {
    exit_syscall(1);
  }

  exit_syscall(42);
}
