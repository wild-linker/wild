//#LinkerDriver:clang
//#SoSingleLinker:wild
//#Shared:tls.c

extern _Thread_local int value;

int main(void) { return value; }
