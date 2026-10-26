//#AbstractConfig:default
//#Object:ifunc-address-equality-1.c:-fPIC
//#Object:ifunc_init.c
//#Object:runtime.c
//#RequiresGlibc:true
//#Arch:x86_64

//#Config:no-pie:default
//#CompArgs:-fno-pie

//#Config:pic:no-pie
// Wild currently uses an extra PLT indirection.
//#DiffIgnore:rel.extra-got-plt-got
//#CompArgs:-fPIC

//#Config:pie-mixed:default
// BFD gives the PIE and PIC callers different IFUNC addresses.
//#ReferenceLinkers:lld
// LLD places IFUNC stubs in .iplt and their relocations in .rela.dyn.
//#DiffIgnore:.dynamic.DT_RELA*
//#DiffIgnore:section.rela.dyn
//#DiffIgnore:section.iplt
//#DiffIgnore:section.got.plt.entsize
//#Mode:dynamic
//#CompArgs:-fPIE -DDYNAMIC
//#LinkArgs:-pie

//#Config:pic-pie:pie-mixed
//#CompArgs:-fPIC
//#DiffIgnore:rel.extra-got-plt-got

//#Config:pie-mixed-data:pie-mixed
//#CompArgs:-DDATA_POINTER

//#Config:pie-mixed-relr:pie-mixed-data
//#LinkArgs:-z pack-relative-relocs
//#ExpectDynamic:DT_RELR
// linker-diff does not resolve RELR-backed IFUNC PLT pointers.
//#DiffIgnore:rel.R_X86_64_REX_GOTPCRELX.R_X86_64_REX_GOTPCRELX
//#DiffIgnore:rel.R_X86_64_64.R_X86_64_64

//#Config:pie-mixed-no-relax:pie-mixed
//#LinkArgs:--no-relax
// LLD binds local calls directly with --no-relax.
//#DiffIgnore:rel.extra-plt-got

#include "../common/ifunc_init.h"
#include "../common/runtime.h"

typedef void (*Func)(void);

__attribute__((ifunc("resolve_foo"))) void foo(void);
static void real_foo(void) {}
static Func resolve_foo(void) { return real_foo; }

__attribute__((ifunc("resolve_bar"))) void bar(void);
static void real_bar(void) {}
static Func resolve_bar(void) { return real_bar; }

#ifdef DATA_POINTER
Func data_foo = foo;
#endif

extern Func get_foo(void);
extern Func get_bar(void);

void _start(void) {
  runtime_init();

#ifndef DYNAMIC
  int rv = init_ifuncs();
  if (rv != 0) {
    exit_syscall(rv);
  }
#endif

  Func direct_foo = foo;
  Func got_foo = get_foo();

  if (direct_foo != got_foo) {
    exit_syscall(1);
  }

  Func direct_bar = bar;
  Func got_bar = get_bar();

  if (direct_bar != got_bar) {
    exit_syscall(2);
  }

#ifdef DATA_POINTER
  if (direct_foo != data_foo) {
    exit_syscall(4);
  }
  data_foo();
#endif

  if (direct_foo == direct_bar) {
    exit_syscall(3);
  }

  direct_foo();
  got_foo();
  direct_bar();
  got_bar();

  exit_syscall(42);
}
