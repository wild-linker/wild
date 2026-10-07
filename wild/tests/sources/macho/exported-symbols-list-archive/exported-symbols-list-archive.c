//#LinkerDriver:clang
//#LinkArgs:-dynamiclib -Wl,-exported_symbols_list,./exported-symbols-list-archive.txt
//#Archive:member.c
//#RunEnabled:false
//#ExpectDynSym:_wild_test_export_from_archive
