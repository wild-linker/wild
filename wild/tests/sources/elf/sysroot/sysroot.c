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
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:library-between-sysroots:base
// LLD 21 does not expand $SYSROOT in library search paths.
//#ReferenceLinkers:bfd
//#LinkArgs:--sysroot=$OUT_DIR/first -L$SYSROOT/lib --sysroot=$OUT_DIR/last
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:library-after-sysroots:base
//#LinkArgs:--sysroot=$OUT_DIR/first --sysroot=$OUT_DIR/last -L=/lib
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-library-before-sysroots:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:-L=/lib --sysroot=$OUT_DIR/first --sysroot=$OUT_DIR/last
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-library-between-sysroots:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:--sysroot=$OUT_DIR/first -L$SYSROOT/lib --sysroot=$OUT_DIR/last
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:library-without-sysroot:base
//#ReferenceLinkers:
//#LinkArgs:-L=$OUT_DIR/last/lib
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-library-without-sysroot:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:-L=$OUT_DIR/last/lib
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:script-base
//#Env:NIX_ENFORCE_PURITY=0
//#ReferenceLinkers:bfd
//#Object:runtime.c
//#Archive:as(first/lib/libvalue.a),noadd:first.c
//#Archive:as(last/lib/libvalue.a),noadd:last.c
//#ExpectSym:last_root
//#NoSym:first_root

//#Config:script-forced-equals:script-base
//#LinkArgs:--sysroot=$OUT_DIR/last
//#AugmentLinkerScript:script-equals.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-script-forced-equals:script-base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:--sysroot=$OUT_DIR/last
//#AugmentLinkerScript:script-equals.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:script-forced-sysroot-var:script-base
//#LinkArgs:--sysroot=$OUT_DIR/last
//#AugmentLinkerScript:script-sysroot-var.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-script-forced-sysroot-var:script-base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkArgs:--sysroot=$OUT_DIR/last
//#AugmentLinkerScript:script-sysroot-var.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:forced-file-equals:script-base
//#LinkArgs:--sysroot=$OUT_DIR/last
//#PostLinkArgs:=/lib/libvalue.a
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:forced-file-sysroot-var:script-base
//#LinkArgs:--sysroot=$OUT_DIR/last
//#PostLinkArgs:$SYSROOT/lib/libvalue.a
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:forced-file-equals-no-sysroot:script-base
//#ReferenceLinkers:
//#PostLinkArgs:=$OUT_DIR/last/lib/libvalue.a
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-forced-file-sysroot-var-no-sysroot:script-base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#PostLinkArgs:$SYSROOT$OUT_DIR/last/lib/libvalue.a
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-forced-file-equals-no-sysroot:script-base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#PostLinkArgs:=$OUT_DIR/last/lib/libvalue.a
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:script-forced-equals-no-sysroot:script-base
//#ReferenceLinkers:
//#AugmentLinkerScript:script-equals-no-sysroot.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-script-forced-equals-no-sysroot:script-base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#AugmentLinkerScript:script-equals-no-sysroot.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:script-forced-sysroot-var-no-sysroot:script-base
//#ReferenceLinkers:
//#AugmentLinkerScript:script-sysroot-var-no-sysroot.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-script-forced-sysroot-var-no-sysroot:script-base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#AugmentLinkerScript:script-sysroot-var-no-sysroot.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

int value(void);

void _start(void) {
  runtime_init();
  exit_syscall(value());
}
