//#AbstractConfig:default
//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#RunEnabled:false

//#Config:restricted:default
// Apply the export list only to the final link, so the provider still exports both symbols.
//#Shared:provider.c
//#AugmentLinkerScript:template(-Wl,-exported_symbols_list,$O):exported-symbols-list-overlap.txt
//#ExpectDynSym:_wild_test_export_overlap
//#NoDynSym:_wild_test_private_overlap
//#ExpectSym:_wild_test_private_overlap section="__private",offset-in-section=0,binding=local

//#Config:unrestricted:default
//#Shared:provider.c
//#ExpectDynSym:_wild_test_export_overlap
//#ExpectDynSym:_wild_test_private_overlap

int wild_test_export_overlap(void) { return 1; }
__attribute__((section("__DATA,__private"))) int wild_test_private_overlap = 2;
