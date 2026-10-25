//#Config:linker-script-assert-fail
//#LinkArgs:-T ./linker-script-assert-fail.ld
//#ExpectError:assertion failed: text section cannot be empty
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:linker-script-assert-symbol-fail
//#LinkArgs:-T ./linker-script-assert-symbol-fail.ld
//#ExpectError:symbol1 must be 0x2000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

void _start() {}
