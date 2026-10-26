//#AbstractConfig:default
//#Arch:x86_64
//#RequiresGlibc:true
//#LinkerDriver:gcc
//#LinkArgs:-pie -Wl,--export-dynamic
//#PostLinkArgs:-ldl
//#ReferenceLinkers:bfd,lld
// Wild uses the PLT address even when all address references use the GOT.
//#DiffIgnore:rel.extra-got-plt-got
//#DiffIgnore:dynsym.foo.section

//#Config:pic:default
//#CompArgs:-fPIC

//#Config:pic-no-relax:pic
//#LinkArgs:-Wl,--no-relax

#include <dlfcn.h>

typedef int (*Func)(void);

static int implementation(void) { return 42; }
static Func resolve_foo(void) { return implementation; }
__attribute__((ifunc("resolve_foo"))) int foo(void);

int main(void) {
  Func dynamic_foo = (Func)dlsym(RTLD_DEFAULT, "foo");
  if (foo != dynamic_foo) {
    return 1;
  }
  return dynamic_foo();
}
