//#LinkerDriver:clang
//#SoSingleLinker:lld
//#Shared:foo.c
//#DiffIgnore:section.__unwind_info
//#NoSection:__got
//#ExpectSym:_ptr section="__data",segment="__DATA"

extern int value;

int* volatile ptr = &value;

int main(void) { return *ptr; }
