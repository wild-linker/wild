//#AbstractConfig:default
//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#RunDynSym:foo
//#ExpectDynSym:_foo
//#ExpectDynSym:_unreferenced
//#ExpectDynSym:_value
//#ExpectDynSym:_pointer
//#NoDynSym:_hidden

//#Config:basic:default

//#Config:install-name:default
//#LinkArgs:-Wl,-install_name,@rpath/libminimal.dylib
//#ExpectIdDylib:@rpath/libminimal.dylib

//#Config:imports:default
//#CompArgs:-DIMPORTS

//#Config:archive:default
//#CompArgs:-DARCHIVE
//#Archive:member.c
//#ExpectDynSym:_member

int value = 42;
int* volatile pointer = &value;

int unreferenced(void) { return 99; }

__attribute__((visibility("hidden"))) int hidden(void) { return 0; }

#ifdef IMPORTS
#include <stdlib.h>
#endif

#ifdef ARCHIVE
int member(void);
#endif

int foo(void) {
#ifdef IMPORTS
  void* allocation = malloc(16);
  if (!allocation) return 1;
  free(allocation);
#endif

#ifdef ARCHIVE
  if (member() != 42) return 2;
#endif

  return *pointer + hidden();
}
