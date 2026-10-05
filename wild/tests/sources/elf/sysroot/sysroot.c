//#AbstractConfig:base
// Nix's linker wrapper otherwise removes --sysroot arguments.
//#Env:NIX_ENFORCE_PURITY=0
//#ReferenceLinkers:bfd,lld
//#Object:runtime.c
//#Archive:as(first/lib/libvalue.a),noadd:first.c
//#Archive:as(last/lib/libvalue.a),noadd:last.c
//#PostLinkArgs:-lvalue
//#ExpectSym:last_root
//#NoSym:first_root

//#Config:library-before-sysroots:base
//#LinkArgs:-L=/lib --sysroot=$OUT_DIR/first --sysroot=$OUT_DIR/last

//#Config:library-between-sysroots:base
// LLD 21 does not expand $SYSROOT in library search paths.
//#ReferenceLinkers:bfd
//#LinkArgs:--sysroot=$OUT_DIR/first -L$SYSROOT/lib --sysroot=$OUT_DIR/last

//#Config:library-after-sysroots:base
//#LinkArgs:--sysroot=$OUT_DIR/first --sysroot=$OUT_DIR/last -L=/lib

//#Config:save-library-before-sysroots:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:-L=/lib --sysroot=$OUT_DIR/first --sysroot=$OUT_DIR/last

//#Config:save-library-between-sysroots:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:--sysroot=$OUT_DIR/first -L$SYSROOT/lib --sysroot=$OUT_DIR/last

#include "../common/runtime.h"

int value(void);

void _start(void) {
  runtime_init();
  exit_syscall(value());
}
