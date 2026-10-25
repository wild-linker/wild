//#LinkArgs:-T ./linker-script-alignof-fail.ld
//#ExpectError:expected .data alignment to be 2048
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

int unaligned_data = 42;
void _start() {}
