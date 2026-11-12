//#Arch:x86_64
//#Compiler:gcc
//#CompArgs:-fpie -Wa,-mrelax-relocations=no
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now
//#LinkArgs:-pie
//#DiffIgnore:section.rodata

// rustc emits plain R_X86_64_GOTPCREL (never GOTPCRELX). Wild relaxes a 64-bit mov through it to
// a lea when the target is non-interposable; GNU ld only relaxes GOTPCRELX, so it keeps the GOT
// load. Both produce the same address.

__attribute__((visibility("hidden"))) int value = 40;

__asm__(
    ".text\n"
    ".globl get_value_ptr\n"
    ".type get_value_ptr, @function\n"
    "get_value_ptr:\n"
    "    movq value@GOTPCREL(%rip), %rax\n"
    "    ret\n"
    ".size get_value_ptr, .-get_value_ptr\n");

int* get_value_ptr(void);

int main() {
  int* p = get_value_ptr();
  if (p != &value) {
    return 1;
  }
  return *p + 2;
}
