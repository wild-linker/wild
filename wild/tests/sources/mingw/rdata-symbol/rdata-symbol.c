//#Object:runtime.c
//#Object:dep.c
//#ReferenceLinkers:bfd,lld
//#ExpectSym: foo section=".rdata"
//#ExpectSym: foo_ptr section=".data"

extern const int* foo_ptr;

int main() { return *foo_ptr; }
