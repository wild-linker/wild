//#LinkerDriver:clang
//#SoSingleLinker:lld
//#Shared:foo.c
//#ExpectSym:_ptr section="__data",segment="__DATA"

extern int values[2];

int* volatile ptr = values + 1;

int main(void) {
  if (ptr != values + 1) return 1;
  return *ptr;
}
