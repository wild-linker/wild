//#AbstractConfig:default
//#RunEnabled:false
// BFD evaluates both operands; LLD short-circuits these expressions.
//#ReferenceLinkers:bfd
//#CompArgs:-fPIE
//#LinkArgs:-no-pie
//#ExpectError:(Division|/) by zero

//#Config:and:default
//#LinkerScript:and.ld

//#Config:or:default
//#LinkerScript:or.ld

void _start(void) {}
