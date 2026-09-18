//#LinkerDriver:clang
//#SoSingleLinker:lld
//#Shared:function.c
//#DiffIgnore:section.__unwind_info
//#ExpectSection:__got

int plus_one(int value);

int main(void) {
  if (plus_one(17) != 18) return 1;
  int (*volatile function)(int) = plus_one;
  return function(41);
}
