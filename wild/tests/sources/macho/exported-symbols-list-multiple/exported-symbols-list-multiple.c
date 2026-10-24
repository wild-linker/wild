//#AbstractConfig:default
//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#RunEnabled:false

//#Config:union:default
//#LinkArgs:-Wl,-exported_symbols_list,./first.txt
//#LinkArgs:-Wl,-exported_symbols_list,./second.txt
//#LinkArgs:-Wl,-exported_symbols_list,./first.txt
//#LinkArgs:-Wl,-exported_symbols_list,./empty.txt
//#Archive:member.c
//#ExpectDynSym:_first
//#ExpectDynSym:_second
//#ExpectDynSym:_shared
//#ExpectDynSym:_pattern_match
//#ExpectDynSym:_from_archive
//#NoDynSym:_hidden

//#Config:empty:default
//#LinkArgs:-Wl,-exported_symbols_list,./empty.txt
//#LinkArgs:-Wl,-exported_symbols_list,./empty.txt
//#NoDynSym:_first
//#NoDynSym:_second
//#NoDynSym:_shared
//#NoDynSym:_pattern_match
//#NoDynSym:_hidden

//#Config:missing:default
//#LinkArgs:-Wl,-exported_symbols_list,./first.txt
//#LinkArgs:-Wl,-exported_symbols_list,./missing.txt
//#ExpectError:_?missing_export

//#Config:malformed:default
//#ReferenceLinkers:lld
//#LinkArgs:-Wl,-exported_symbols_list,./first.txt
//#LinkArgs:-Wl,-exported_symbols_list,./malformed.txt
//#ExpectError:(Failed to parse export list .*malformed.txt|invalid symbol-name pattern)
//#ExpectErrorWild:Failed to parse symbol export list line 2

//#Config:unrestricted:default
//#ExpectDynSym:_first
//#ExpectDynSym:_second
//#ExpectDynSym:_hidden

int first(void) { return 1; }
int second(void) { return 2; }
int shared(void) { return 3; }
int pattern_match(void) { return 4; }
int hidden(void) { return 5; }
