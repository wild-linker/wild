//#AbstractConfig:default
//#LinkerDriver:clang
//#SoSingleLinker:wild

//#Config:default-name:default
//#Shared:foo.c

//#Config:install-name:default
//#Shared:as(runtime/libminimal.dylib):foo.c
//#LinkSoArgs:-Wl,-install_name,@rpath/libminimal.dylib
//#LinkArgs:-Wl,-rpath,@loader_path/runtime
//#ExpectLoadDylib:@rpath/libminimal.dylib

int foo(void);
extern int value;
int main(void) { return value == 42 ? foo() : 1; }
