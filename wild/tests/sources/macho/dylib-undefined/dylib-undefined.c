//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#ExpectError:(?s)(symbol.*missing|missing.*symbol)

int missing(void);
int foo(void) { return missing(); }
