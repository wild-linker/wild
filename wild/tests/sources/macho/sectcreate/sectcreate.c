//#LinkerDriver:clang
//#Copy:info.txt:info.txt
//#LinkArgs:-Wl,-sectcreate,__TEXT,__info,$OUT_DIR/info.txt
//#ExpectSectionBytes:__info=0x68656c6c6f2073656374637265617465

#include <stdlib.h>

int main(void) { exit(42); }
