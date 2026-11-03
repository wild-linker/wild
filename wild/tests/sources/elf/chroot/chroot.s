//#AbstractConfig:settings
//#Arch:x86_64
//#ReferenceLinkers:lld
//#AutoAddObjects:false
//#LinkerWorkingDirectory:$OUT_DIR
//#Mode:static
//#ExpectSym:_start

//#AbstractConfig:base:settings
//#Object:as(root/entry.o):entry.s

//#Config:absolute:base
//#LinkArgs:--chroot root /entry.o

//#Config:late-option:base
//#LinkArgs:/entry.o --chroot root

//#Config:relative:base
//#LinkArgs:--chroot root root/entry.o

//#Config:script:base
//#Copy:script.txt:root/script
//#LinkArgs:--chroot root /script

//#Config:duplicate-script-input:base
//#ReferenceLinkers:
//#Copy:script.txt:root/script
//#LinkArgs:--chroot root /entry.o /script

//#Config:save-script:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Copy:script.txt:root/script
//#LinkArgs:--chroot root /script

//#Config:save-script-absolute-root:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Copy:script.txt:root/script
//#LinkArgs:--chroot $OUT_DIR/root /script

//#Config:nested-script:base
//#Copy:nested.txt:root/script
//#Copy:script.txt:root/inner
//#LinkArgs:--chroot root /script

//#Config:save-nested-script:nested-script
//#ReferenceLinkers:
//#DriverMode:save-dir-response

//#Config:retain:base
//#Copy:retain.txt:root/retain
//#LinkArgs:--retain-symbols-file /retain /entry.o --chroot root

//#Config:response:base
//#Copy:args.rsp:args.rsp
//#LinkArgs:@$OUT_DIR/args.rsp

//#Config:archive:base
//#Archive:as(root/libentry.a):entry.s
//#LinkArgs:--chroot root -Lroot -lentry -u _start

//#Config:exact-library:base
//#Archive:as(root/libentry.a):entry.s
//#LinkArgs:--chroot root -Lroot -l:libentry.a -u _start

//#Config:version-script:base
//#ExpectSym:discard_me binding=local
//#Copy:versions.txt:root/versions
//#LinkArgs:--chroot root /entry.o --version-script root/versions

//#Config:dynamic-list:base
//#Copy:exports.txt:root/exports
//#LinkArgs:--chroot root /entry.o --dynamic-list /exports

//#Config:missing:base
//#LinkArgs:--chroot root $OUT_DIR/root/entry.o
//#ExpectError:entry.o

//#Config:absolute-root:base
//#LinkArgs:--chroot $OUT_DIR/root /entry.o

//#Config:last-root:base
//#LinkArgs:--chroot nonexistent --chroot root /entry.o

//#Config:relative-script:base
//#Copy:relative.txt:root/script
//#LinkArgs:--chroot root /script

//#Config:sysroot:settings
// LLD tests the unrooted script path for sysroot membership.
//#ReferenceLinkers:
//#Object:as(root/sysroot/entry.o):entry.s
//#Copy:script.txt:root/sysroot/script
//#LinkArgs:--chroot root --sysroot /sysroot /sysroot/script

//#Config:forced-sysroot:settings
//#Object:as(root/sysroot/entry.o):entry.s
//#Copy:sysroot.txt:root/script
//#LinkArgs:--sysroot /sysroot /script --chroot root

//#Config:thin-relative:base
//#ThinArchive:entry.s
//#LinkArgs:--chroot root entry.a -u _start

//#Config:absolute-version-script:base
//#ExpectSym:discard_me binding=local
// LLD checks the unrooted version-script path before reading it.
//#ReferenceLinkers:
//#Copy:versions.txt:root/versions
//#LinkArgs:--chroot root /entry.o --version-script /versions

//#Config:absolute-library-search:base
// LLD checks the unrooted library search path before reading the library.
//#ReferenceLinkers:
//#Archive:as(root/libentry.a):entry.s
//#LinkArgs:--chroot root -L/ -lentry -u _start

//#Config:relative-sysroot:settings
//#Object:as(root/sysroot/entry.o):entry.s
//#Copy:script.txt:root/sysroot/script
//#LinkArgs:--sysroot root/sysroot --chroot root root/sysroot/script

//#Config:thin-relocated:base
//#ThinArchive:entry.s
//#Object:as(root/entry.s.o):entry.s
//#Copy:$OUT_DIR/entry.a:root/entry.a
//#LinkArgs:--chroot root /entry.a -u _start

//#Config:equals:base
// LLD accepts only the separate-argument spelling.
//#ReferenceLinkers:
//#LinkArgs:--chroot=root /entry.o

//#Config:thin-absolute:base
// LLD opens absolute thin-archive members without applying the virtual root.
//#ReferenceLinkers:
//#Copy:absolute.thin:root/absolute.a
//#LinkArgs:--chroot root --whole-archive /absolute.a

//#Config:empty-root:base
//#ReferenceLinkers:
//#LinkArgs:--chroot nonexistent --chroot= root/entry.o

//#Config:save-relative:relative
//#ReferenceLinkers:
//#DriverMode:save-dir-response

//#Config:save-library:archive
//#ReferenceLinkers:
//#DriverMode:save-dir-response

//#Config:save-absolute-library:absolute-library-search
//#DriverMode:save-dir-response

//#Config:save-equals:equals
//#DriverMode:save-dir-response

//#Config:save-last-root:last-root
//#ReferenceLinkers:
//#DriverMode:save-dir-response

//#Config:save-empty-root:empty-root
//#DriverMode:save-dir-response

//#Config:save-sysroot:sysroot
//#DriverMode:save-dir-response

//#Config:save-relative-sysroot:relative-sysroot
//#ReferenceLinkers:
//#DriverMode:save-dir-response

//#Config:save-sysroot-library:settings
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Archive:as(root/sysroot/libentry.a):entry.s
//#LinkArgs:--chroot root --sysroot /sysroot -L=/ -lentry -u _start

//#Config:save-version-script:absolute-version-script
//#DriverMode:save-dir-response

//#Config:save-version-script-equals:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#ExpectSym:discard_me binding=local
//#Copy:versions.txt:root/versions
//#LinkArgs:--chroot root /entry.o --version-script=/versions

//#Config:save-response:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Copy:args.rsp:args.rsp
//#LinkArgs:@$OUT_DIR/args.rsp

//#Config:save-retain:retain
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#NoSym:discard_me

//#Config:save-dynamic-list:dynamic-list
//#ReferenceLinkers:
//#DriverMode:save-dir-response

//#Config:save-option-value:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Mode:unspecified
//#RunEnabled:false
//#LinkArgs:-shared --chroot root /entry.o -soname --chroot=/entry.o
//#Contains:--chroot=/entry.o

//#Config:save-output-prefix-value:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#Mode:unspecified
//#RunEnabled:false
//#LinkArgs:-shared --chroot root /entry.o -soname -output-name
//#Contains:-output-name
