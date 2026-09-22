//#LinkerDriver:clang
// LLD gives the symbol an incorrect value.
//#ReferenceLinkers:ld
//#ExpectDynSym:_absolute_symbol address=42

__asm__(
    ".globl _absolute_symbol\n"
    "_absolute_symbol = 42\n");

int main(void) { return 42; }
