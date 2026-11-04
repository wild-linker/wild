// Checks that the dynamic loader calls the functions that -init and -fini name for a shared object:
// lib_init before main runs, and lib_fini at exit, where it sets the exit code.
//#AbstractConfig:base
//#LinkerDriver:gcc
//#Shared:init-fini-run-lib.c
//#CompArgs:-fPIC
//#CompSoArgs:-fPIC
//#LinkArgs:-pie
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:section.rodata
// lib_init and lib_fini are in the middle of .text, at layout-dependent addresses, so linker-diff
// can't match the tag values between linkers. Running the program checks them instead.
//#DiffIgnore:.dynamic.DT_INIT
//#DiffIgnore:.dynamic.DT_FINI

//#Config:direct:base
//#LinkSoArgs:-Wl,-init,lib_init -Wl,-fini,lib_fini

// -init and -fini can also name --defsym aliases, which resolve to absolute values.
//#Config:defsym:base
//#LinkSoArgs:-Wl,--defsym=init_hook=lib_init -Wl,--defsym=fini_hook=lib_fini
//#LinkSoArgs:-Wl,-init,init_hook -Wl,-fini,fini_hook

//#Config:defsym-absolute:base
//#LinkSoArgs:-Wl,--defsym=init_hook=ABSOLUTE(lib_init)
//#LinkSoArgs:-Wl,--defsym=fini_hook=ABSOLUTE(lib_fini)
//#LinkSoArgs:-Wl,-init,init_hook -Wl,-fini,fini_hook

int lib_init_ran(void);

int main(void) { return lib_init_ran() ? 0 : 1; }
