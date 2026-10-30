//#AbstractConfig:base
//#LinkerDriver:clang
//#LinkArgs:-nostdlib -dynamiclib
//#LinkerWorkingDirectory:$OUT_DIR
//#Archive:as(sdk/usr/lib/libvalue.a),noadd:value.c
// No system symbols are used, but ld and dyld require a dependency on libSystem.
//#Copy:libSystem.tbd:sdk/usr/lib/libSystem.tbd
//#PostLinkArgs:-lSystem
//#RunDynSym:foo
//#ExpectSym:_value
//#ExpectLoadDylib:/usr/lib/libSystem.B.dylib

//#Config:relative:base
//#LinkArgs:-Wl,-syslibroot,sdk
//#PostLinkArgs:-lvalue

//#Config:save-relative:relative
//#DriverMode:save-dir-response

// Resolve the symlink before `..`: lexical simplification would look under aliases/sdk.
//#Config:symlink-parent:base
//#Symlink:sdk:aliases/root
//#PostLinkArgs:-Wl,-syslibroot,aliases/root/../sdk -lvalue

//#Config:save-symlink-parent:symlink-parent
//#DriverMode:save-dir-response

// A nonexistent sysroot is allowed when the library can be found through an explicit -L.
//#Config:missing:base
//#LinkArgs:-Wl,-syslibroot,missing,-L,$OUT_DIR/sdk/usr/lib
//#PostLinkArgs:-lvalue

int value(void);

int foo(void) { return value(); }
