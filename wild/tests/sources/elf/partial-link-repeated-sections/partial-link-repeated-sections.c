// Test that partial linking merges sections with the same name.

//#AbstractConfig:base
//#Object:partial-link-repeated-sections-2.c
//#RunEnabled:false
//#ReferenceLinkers:bfd,lld
//#DiffEnabled:false
//#ExpectSectionBytes:foo=0x1122
//#ExpectSectionBytes:.bar=0x3344
//#CompArgs:-fPIE

//#Config:default:base
//#LinkArgs:-r

//#Config:gcc:base
//#LinkerDriver:gcc
//#LinkArgs:-nostdlib -no-pie -Wl,-r,-z,now

//#Config:clang:base
//#Compiler:clang
//#LinkerDriver:clang
//#LinkArgs:-nostdlib -no-pie -Wl,-z,now,--relocatable

const unsigned char foo_a __attribute__((section("foo"), used)) = 0x11;
const unsigned char bar_a __attribute__((section(".bar"), used)) = 0x33;
