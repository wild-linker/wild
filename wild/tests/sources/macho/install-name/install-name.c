//#AbstractConfig:default
//#LinkerDriver:clang
//#SoSingleLinker:lld
//#LinkSoArgs:-Wl,-install_name,@rpath/libinstall-name.dylib
//#LinkArgs:-Wl,-rpath,@loader_path/runtime
//#ExpectLoadDylib:@rpath/libinstall-name.dylib

//#Config:thin:default
//#Shared:as(runtime/libinstall-name.dylib):foo.c

//#Config:fat:default
//#Shared:as(runtime/libinstall-name.dylib),noadd:foo.c
//#FatDylib:foo.c

//#Config:fat64:default
//#ReferenceLinkers:
//#Shared:as(runtime/libinstall-name.dylib),noadd:foo.c
//#FatDylib64:foo.c

//#Config:duplicate:default
// Both dylibs will have an install name of `@rpath/libinstall-name.dylib`. We'll resolve bar from
// the first and foo from the second, but at runtime only the second file, which has a path matching
// the install name will be used.
//#Shared:as(other/libalias.dylib):alias.c
//#Shared:as(runtime/libinstall-name.dylib):foo.c

int foo(void);
int bar(void);

int main(void) { return bar() == 22 ? foo() : 1; }
