//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#RunEnabled:false
//#ExpectSym:_private_data section="__private",offset-in-section=0,binding=local

__attribute__((used, section("__DATA,__private"))) static int private_data = 42;

int exported_function(void) { return 0; }
