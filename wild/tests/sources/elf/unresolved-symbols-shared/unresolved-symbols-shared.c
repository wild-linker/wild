//#AbstractConfig:default
//#ReferenceLinkers:
//#DiffEnabled:false
//#Mode:dynamic
//#RunEnabled:false
//#LinkArgs:-shared -z now

//#Config:ignore-in-object-files:default
//#LinkArgs:--unresolved-symbols=ignore-in-object-files

//#Config:ignore-in-shared-libs:default
//#LinkArgs:--unresolved-symbols=ignore-in-shared-libs

int foo();

void _start(void) { foo(); }
