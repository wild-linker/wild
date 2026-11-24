// Checks that the functions named by -init and -fini survive LTO. They're hidden in
// the shared object, so only DT_INIT and DT_FINI refer to them, and LTO would otherwise be
// free to discard them.
//#AbstractConfig:default
// glibc only calls DT_INIT on architectures that define ELF_INITFINI. Newer ports such as RISC-V
// and LoongArch only support .init_array.
//#Arch:x86_64,aarch64
//#RequiresLinkerPlugin:true
//#CompArgs:-fPIC
//#CompSoArgs:-fPIC -flto
//#LinkArgs:-pie
//#LinkSoArgs:-flto -Wl,-init,lib_init -Wl,-fini,lib_fini
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:section.rodata
// lib_init and lib_fini are at layout-dependent addresses. Running the program checks them.
//#DiffIgnore:.dynamic.DT_INIT
//#DiffIgnore:.dynamic.DT_FINI

//#AbstractConfig:weak:default
// The native weak definitions precede the strong LTO definitions, so retention must use the
// selected definitions rather than the first symbols found by name.
//#Shared:init-fini-lto-weak.c:-fno-lto,init-fini-lto-lib.c

// GNU ld discards lib_init and lib_fini here, so there's no reference linker.
//#Config:gcc:weak
//#Compiler:gcc
//#LinkerDriver:gcc
//#ReferenceLinkers:

//#Config:clang:weak
//#Compiler:clang
//#LinkerDriver:clang
//#ReferenceLinkers:lld

//#Config:llvm-full:default
// Prevent Clang from passing a linker plugin for the bitcode inputs.
//#LinkSoArgs:-fno-lto
//#Compiler:clang
//#LinkerDriver:clang
//#RequiresLinkerPlugin:false
//#RequiresLlvmLto:true
//#ReferenceLinkers:lld
//#Shared:init-fini-lto-lib.c

//#Config:llvm-thin:llvm-full
//#CompSoArgs:-flto=thin

int lib_init_ran(void);

int main(void) { return lib_init_ran() ? 0 : 1; }
