//#LinkerDriver:clang
//#LinkArgs:-dynamiclib -Wl,-exported_symbols_list,./exported-symbols-list-missing.txt
//#RunEnabled:false
//#ExpectError:_?wild_test_missing_export
