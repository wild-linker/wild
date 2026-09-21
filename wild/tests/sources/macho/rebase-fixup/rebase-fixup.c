//#LinkerDriver:clang

char target[9] = "012345679";
char* pointer = target + 0x5;

int main(void) {
  if (pointer != target + 5) return 1;
  if (*pointer != '5') return 2;
  return 42;
}
