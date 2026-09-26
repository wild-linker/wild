//#Arch:aarch64
//#CompArgs:-O2
//#LinkerDriver:clang

int value = 1;
extern long difference;

__asm__(
    ".data\n"
    ".p2align 3\n"
    ".globl _difference\n"
    "_difference: .quad _value - _difference + 7\n");

int main(void) { return difference == (long)&value - (long)&difference + 7 ? 42 : 1; }
