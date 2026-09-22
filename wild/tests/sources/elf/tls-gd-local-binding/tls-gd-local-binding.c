//#AbstractConfig:default
//#Arch:x86_64,riscv64
//#RequiresGlibc:true
//#LinkerDriver:gcc
//#CompSoArgs:-fPIC -ftls-model=global-dynamic
//#LinkArgs:-Wl,-z,now -pie
//#Shared:tls-gd-local-binding-lib.c
//#ReferenceLinkers:bfd,lld
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:.dynamic.DT_RELA*
//#DiffIgnore:section.rodata

//#Config:protected:default
//#CompSoArgs:-DPROTECTED=1

//#Config:symbolic:default
//#LinkSoArgs:-Wl,-Bsymbolic
//#DiffIgnore:.dynamic.DT_FLAGS.SYMBOLIC

//#Config:protected-loongarch64:protected
//#Arch:loongarch64
//#CompSoArgs:-mtls-dialect=trad

//#Config:symbolic-loongarch64:symbolic
//#Arch:loongarch64
//#CompSoArgs:-mtls-dialect=trad

// LLD doesn't support traditional AArch64 TLSGD relocations.
//#Config:protected-aarch64:protected
//#Arch:aarch64
//#CompSoArgs:-mtls-dialect=trad
//#ReferenceLinkers:bfd

//#Config:symbolic-aarch64:symbolic
//#Arch:aarch64
//#CompSoArgs:-mtls-dialect=trad
//#ReferenceLinkers:bfd
//#DiffIgnore:.dynamic.DT_SYMBOLIC

int entry(void);

int main(void) { return entry(); }
