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

//#AbstractConfig:script-contained:script-base
//#ReferenceLinkers:bfd,lld
//#Copy:script-absolute.ld:last/lib/input.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:script-contained-absolute:script-contained
//#PostLinkArgs:$OUT_DIR/last/lib/input.ld
//#LinkArgs:--sysroot=$OUT_DIR/last

//#Config:script-contained-relative:script-contained
//#PostLinkArgs:$OUT_DIR/last/lib/input.ld
//#LinkerWorkingDirectory:$OUT_DIR
//#LinkArgs:--sysroot=last

//#Config:script-contained-dot:script-contained
//#PostLinkArgs:$OUT_DIR/last/lib/input.ld
//#LinkerWorkingDirectory:$OUT_DIR/last
//#LinkArgs:--sysroot=.

//#Config:script-contained-parent:script-contained
//#PostLinkArgs:$OUT_DIR/last/lib/input.ld
//#LinkArgs:--sysroot=$OUT_DIR/last/lib/..

//#Config:script-contained-symlink:script-contained
//#PostLinkArgs:$OUT_DIR/last/lib/input.ld
//#Symlink:last:root-link
//#LinkArgs:--sysroot=$OUT_DIR/root-link

//#AbstractConfig:repeated-base
//#Env:NIX_ENFORCE_PURITY=0
//#ReferenceLinkers:bfd,lld
//#Object:runtime.c
//#Mode:dynamic
//#CompArgs:-fPIC
//#CompSoArgs:-fPIC
//#LinkArgs:-no-pie
//#SoSingleLinker:ld
//#LinkSoArgs:-soname libvalue.so
//#Shared:as(last/lib/libvalue.so),noadd:last.c
//#ExpectDynamic:DT_NEEDED count=1
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT

//#AbstractConfig:repeated-library:repeated-base
//#AugmentLinkerScript:as(last/lib/shared.ld),template(--no-as-needed -L=/lib -lvalue $O -rpath $OUT_DIR/last/lib):script-shared.ld

//#Config:repeated-library-symlink:repeated-library
//#Symlink:last:root-link
//#LinkArgs:--sysroot=$OUT_DIR/root-link

//#Config:repeated-library-parent:repeated-library
//#LinkArgs:--sysroot=$OUT_DIR/last/lib/..

//#Config:repeated-file-symlink:repeated-base
// LLD does not expand = in command-line input paths.
//#ReferenceLinkers:bfd
//#Symlink:last:root-link
//#LinkArgs:--sysroot=$OUT_DIR/root-link
//#AugmentLinkerScript:as(last/lib/shared.ld),template(--no-as-needed =/lib/libvalue.so $O -rpath $OUT_DIR/last/lib):script-shared.ld

//#Config:repeated-library-save:repeated-library
//#ReferenceLinkers:
//#SoSingleLinker:wild
//#DriverMode:save-dir-response
//#Symlink:last:root-link
//#LinkArgs:--sysroot=$OUT_DIR/root-link

//#Config:version-script-search:script-base
//#Copy:version.map:last/lib/version.map
//#Symlink:last:root-link
//#LinkArgs:--sysroot=$OUT_DIR/root-link -L=/lib --version-script=version.map
//#PostLinkArgs:-lvalue
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:save-script-contained-relative:script-contained
//#PostLinkArgs:$OUT_DIR/last/lib/input.ld
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#LinkerWorkingDirectory:$OUT_DIR
//#LinkArgs:--sysroot=last

//#Config:save-script-contained-symlink:script-contained
//#PostLinkArgs:=/lib/input.ld
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Symlink:last:root-link
//#LinkArgs:--sysroot=$OUT_DIR/root-link

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
