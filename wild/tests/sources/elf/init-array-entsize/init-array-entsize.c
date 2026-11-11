// clang gives .init_array an entry size of 0, while crtbeginS.o uses 8. lld keeps the input entry
// size, so it uses 0 when inputs disagree, whereas Wild (like GNU ld) uses 8. Checks that
// linker-diff doesn't report this, and that the constructor runs.
//#Compiler:clang
//#LinkerDriver:gcc
//#ReferenceLinkers:lld
//#CompArgs:-fPIE
//#LinkArgs:-pie
//#DiffIgnore:section.rodata

static int ran;

__attribute__((constructor)) static void init(void) { ran = 1; }

int main(void) { return ran ? 42 : 1; }
