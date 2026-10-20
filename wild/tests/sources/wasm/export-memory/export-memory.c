//#Config:no-option
// Note the __stack_pointer (in Global section) used to be present
// in older LLVM releases: (https://github.com/llvm/llvm-project/pull/199739).
//#ExpectSection: Memory
//#ExpectSym: memory
//#ExpectSym: _start

//#Config:default-value
//#LinkArgs: --export-memory
//#ExpectSym: memory
//#ExpectSym: _start

//#Config:custom-name
//#LinkArgs: --export-memory=foo
//#ExpectSym: foo
//#ExpectSym: _start
//#NoSym: memory

void _start(void) {}
