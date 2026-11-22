// Checks that a compressed section whose data doesn't decompress to exactly the size in its
// compression header is an error, rather than being zero-filled or truncated. Each input's stream
// holds 16 bytes.
// lld accepts these inputs, so only GNU ld is used as a reference.
//#AbstractConfig:default
//#Arch:x86_64,aarch64
//#LinkArgs:-r
//#ReferenceLinkers:bfd

//#Config:zlib-bigger:default
//#Object:zlib-bigger.s
//#ExpectError:header's size of 4096 bytes|bad value

//#Config:zlib-smaller:default
//#Object:zlib-smaller.s
//#ExpectError:header's size of 8 bytes|bad value

//#Config:zstd-bigger:default
//#Object:zstd-bigger.s
// GNU ld isn't always built with zstd support, and when it is, it accepts a zstd stream that's
// shorter than its header says, so the zstd configs have no reference linker.
//#ReferenceLinkers:
//#ExpectError:header's size of 4096 bytes

//#Config:zstd-smaller:default
//#Object:zstd-smaller.s
//#ReferenceLinkers:
//#ExpectError:header's size of 8 bytes

.section .note.GNU-stack,"",%progbits
