//#Object:runtime.c
//#NoSym:__ImageBase

extern char __ImageBase[];

int main() {
  // Check magic bytes
  if (__ImageBase[0] != 'M') return 1;
  if (__ImageBase[1] != 'Z') return 2;

  return 42;
}
