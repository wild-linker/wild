//#AbstractConfig:default
//#LinkerDriver:clang
// Follow Apple ld: LLD also re-exports wildcard matches in unused dependency symbols.
//#ReferenceLinkers:ld
//#LinkArgs:-dynamiclib
//#Shared:provider.c
//#RunEnabled:false

//#Config:exact:default
//#ReferenceLinkers:
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):exported-symbols-list-reexport.txt
//#ExpectErrorWild:re-exporting symbols.*_wild_test_reexport_from_dylib

//#Config:wildcard:default
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):wildcard.txt
//#ExpectDynSym:_wild_test_reexport_local
//#NoDynSym:_wild_test_reexport_from_dylib

//#Config:all:default
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):all.txt
//#ExpectDynSym:_wild_test_reexport_local
//#NoDynSym:_wild_test_reexport_from_dylib

//#Config:imported-wildcard:default
//#ReferenceLinkers:
//#CompArgs:-DIMPORT_SYMBOL
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):wildcard.txt
//#ExpectErrorWild:re-exporting symbols.*_wild_test_reexport_from_dylib

// Wildcards ignore unused dependency symbols, but re-export matching imports.
#ifdef IMPORT_SYMBOL
int wild_test_reexport_from_dylib(void);
int wild_test_reexport_local(void) { return wild_test_reexport_from_dylib(); }
#else
int wild_test_reexport_local(void) { return 42; }
#endif
