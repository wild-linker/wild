//#LinkerDriver:clang
//#SoSingleLinker:lld
//#Shared:foo.c
//#NoSection:__got
//#ExpectSym:_ptr section="__data",segment="__DATA"

extern int value;
extern const unsigned char bytes[1024];

int* volatile ptr = &value;
const unsigned char* volatile ptr16 = bytes + 16;
const unsigned char* volatile ptr255 = bytes + 255;

// TODO: requires DYLD_CHAINED_IMPORT_ADDEND fixup format
// const unsigned char* volatile ptr1023 = bytes + 1023;

int main(void) {
  if (*ptr16 != 16 || *ptr255 != 255) return 1;
  return *ptr;
}
