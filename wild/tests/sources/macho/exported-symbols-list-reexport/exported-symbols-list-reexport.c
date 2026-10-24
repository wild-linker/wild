//#AbstractConfig:default
//#LinkerDriver:clang
// Follow Apple ld: LLD also re-exports wildcard matches in unused dependency symbols.
//#ReferenceLinkers:ld
//#LinkArgs:-dynamiclib
//#Shared:provider.c
//#RunEnabled:false

//#Config:exact:default
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):exported-symbols-list-reexport.txt
//#ExpectDynSym:_wild_test_reexport_from_dylib reexport=1

//#Config:exact-dead-strip:exact
//#LinkArgs:-Wl,-dead_strip_dylibs

//#Config:wildcard:default
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):wildcard.txt
//#ExpectDynSym:_wild_test_reexport_local
//#NoDynSym:_wild_test_reexport_from_dylib

//#Config:all:default
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):all.txt
//#ExpectDynSym:_wild_test_reexport_local
//#NoDynSym:_wild_test_reexport_from_dylib

//#Config:imported-wildcard:default
//#CompArgs:-DIMPORT_SYMBOL
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):wildcard.txt
//#ExpectDynSym:_wild_test_reexport_from_dylib reexport=1

//#Config:stub-dead-strip:default
//#LinkArgs:-Wl,-dead_strip_dylibs
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):stub.txt
//#ExpectDynSym:_malloc reexport=1

//#Config:multiple-providers:default
//#Shared:second.c
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):multiple.txt
//#ExpectDynSym:_wild_test_reexport_from_dylib reexport=1
//#ExpectDynSym:_wild_test_reexport_second reexport=2

// Wildcards ignore unused dependency symbols, but re-export matching imports.
#ifdef IMPORT_SYMBOL
int wild_test_reexport_from_dylib(void);
int wild_test_reexport_local(void) { return wild_test_reexport_from_dylib(); }
#else
int wild_test_reexport_local(void) { return 42; }
#endif
