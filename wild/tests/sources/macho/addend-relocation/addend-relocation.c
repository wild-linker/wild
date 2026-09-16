//#Arch:aarch64
//#CompArgs:-O2
//#LinkerDriver:clang
//#DiffIgnore:section.__unwind_info

int values[4] = {1, 2, 3, 42};

int* offset_pointer(void) { return &values[3]; }

int main() { return *offset_pointer(); }
