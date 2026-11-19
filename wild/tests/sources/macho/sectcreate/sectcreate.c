//#AbstractConfig:base
//#LinkerDriver:clang
//#DiffIgnore:section.__info.attributes

//#Config:default:base
//#Copy:info.txt:info.txt
//#LinkArgs:-Wl,-sectcreate,__TEXT,__info,$OUT_DIR/info.txt
//#ExpectSectionBytes:__info=0x68656c6c6f20736563746372656174650a

//#Config:empty:base
//#Copy:empty.txt:empty.txt
//#LinkArgs:-Wl,-sectcreate,__TEXT,__info,$OUT_DIR/empty.txt
//#ExpectSectionBytes:__info=0x

//#Config:zerofill:base
//#DiffIgnore:section.__bss.attributes
//#Object:zerofill.s
// LLD create a completely invalid binary file
//#ReferenceLinkers:ld
//#Copy:info.txt:info.txt
//#LinkArgs:-Wl,-sectcreate,__DATA,__bss,$OUT_DIR/info.txt
//#ExpectSection:__bss size=21
//#ExpectSym:_sectcreate_bss section="__bss",segment="__DATA"

#include <stdlib.h>

int main(void) { exit(42); }
