//#LinkerDriver:clang
//#SoSingleLinker:wild
//#LinkSoArgs:-Wl,-dead_strip_dylibs
//#Shared:as(libforward.dylib),noadd:forward.c

// Exercise dyld's lookup of emitted re-exports. Linking against re-exports in
// input dylibs is separate functionality.
#include <dlfcn.h>
#include <stdio.h>

int main(void) {
  void* library = dlopen("@loader_path/libforward.dylib", RTLD_NOW | RTLD_LOCAL | RTLD_FIRST);
  if (!library) {
    fprintf(stderr, "%s\n", dlerror());
    return 1;
  }
  int (*forwarded)(void) = dlsym(library, "wild_test_forwarded");
  if (!forwarded || forwarded() != 42) return 2;
  // RTLD_FIRST must not expose ordinary exports of the provider.
  if (dlsym(library, "wild_test_not_forwarded")) return 3;
  return dlclose(library) == 0 ? 42 : 4;
}
