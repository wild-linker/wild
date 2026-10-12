//#Arch:ppc64le
//#LinkerDriver:gcc
//#DiffEnabled:false
//#Shared:lib.c
//#CompSoArgs:-fPIC -O2

int add(int a, int b);

int main(void) { return add(20, 22); }
