//#AbstractConfig:base
//#Arch:x86_64
//#Mode:dynamic
//#CompArgs:-fPIC
//#CompSoArgs:-fPIC
// The first occurrence of each version marker belongs to a discarded DSO.
//#Shared:template(--push-state --as-needed $O --pop-state):unused.c
//#Shared:libc.c
// BFD emits the version-marker symbols that glibc exports.
//#SoSingleLinker:ld
//#LinkArgs:-shared -z mark-plt
//#LinkSoArgs:-soname libc.so.6
//#RunEnabled:false
//#ReferenceLinkers:bfd
//#RequiresLinkerFlags:-z mark-plt
//#DiffIgnore:.dynamic.DT_X86_64_PLTSZ
//#DiffIgnore:.dynamic.DT_X86_64_PLT
//#DiffIgnore:.dynamic.DT_RELA*
//#DiffIgnore:section.got

//#Config:abi:base
// Older BFD versions always require GLIBC_2.36.
//#ReferenceLinkers:
//#LinkSoArgs:--version-script ./abi.map
//#Contains:GLIBC_ABI_DT_X86_64_PLT
//#DoesNotContain:GLIBC_2.36

//#Config:fallback:base
//#LinkSoArgs:--version-script ./fallback.map
//#Contains:GLIBC_2.36
//#DoesNotContain:GLIBC_ABI_DT_X86_64_PLT

//#Config:old:base
//#LinkSoArgs:--version-script ./old.map
//#ReferenceLinkers:
//#ExpectError:-z mark-plt requires libc.so.6 to define GLIBC_ABI_DT_X86_64_PLT or GLIBC_2.36

//#Config:disabled:base
//#LinkSoArgs:--version-script ./old.map
//#LinkArgs:-z nomark-plt
//#DoesNotContain:GLIBC_ABI_DT_X86_64_PLT
//#DoesNotContain:GLIBC_2.36

//#Config:unused:base
//#LinkSoArgs:--version-script ./old.map
//#LinkArgs:--as-needed
//#CompArgs:-DNO_REFERENCE
//#DoesNotContain:libc.so.6
//#DoesNotContain:GLIBC_
//#NoDynamic:DT_X86_64_PLT
//#NoDynamic:DT_X86_64_PLTSZ
//#NoDynamic:DT_X86_64_PLTENT

extern int foo(void);

int call_foo(void) {
#ifdef NO_REFERENCE
  return 42;
#else
  return foo();
#endif
}
