// wasm-ld does not support --chroot.
//#ReferenceLinkers:
//#AutoAddObjects:false
//#Object:as(root/input.o),noadd:chroot.c
//#LinkArgs:--chroot $OUT_DIR/root /input.o
//#ExpectSym:_start

void _start(void) {}
