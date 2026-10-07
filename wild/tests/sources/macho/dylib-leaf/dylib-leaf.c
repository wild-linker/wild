//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#RunDynSym:foo
//#ExpectDynSym:_foo

int foo(void) { return 42; }
