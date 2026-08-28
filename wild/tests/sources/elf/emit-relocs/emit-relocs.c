//#CompArgs:-fPIE -ffunction-sections
//#Object:runtime.c
//#LinkArgs:-no-pie --emit-relocs
//#ExpectSection:.rela.text
//#ExpectSection:.rela.custom.init
//#NoSection:.rela.text._start
//#NoSection:.rela.text.foo
//#NoSection:.rela.custom
//#ExpectSym:_start section=".text"
//#ExpectSym:foo section=".text"
//#ExpectSym:exit_syscall section=".text"
//#DiffIgnore:rel.reference_failed.*

#include "../common/runtime.h"

int foo(void) { return 42; }

__attribute__((section(".custom.init"), used)) void init(void) { runtime_init(); }

void _start(void) {
  init();
  exit_syscall(foo());
}
