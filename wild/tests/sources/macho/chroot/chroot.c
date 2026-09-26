//#AbstractConfig:base
// ld64.lld does not support --chroot.
//#ReferenceLinkers:
//#RunEnabled:false
//#AutoAddObjects:false
//#ExpectEntry:_main
//#ExpectLoadDylib:/usr/lib/libSystem.B.dylib
//#Copy:libSystem.tbd:root/sdk/usr/lib/libSystem.tbd
//#LinkArgs:-syslibroot /sdk

//#Config:absolute:base
//#Object:as(root/input.o),noadd:chroot.c
//#LinkArgs:--chroot $OUT_DIR/root /input.o

//#Config:relative-sysroot:base
//#LinkerWorkingDirectory:$OUT_DIR
//#Archive:as(root/sdk/usr/lib/libmain.a),noadd:chroot.c
//#LinkArgs:--chroot root -syslibroot root/sdk -lmain

int main(void) { return 42; }
