//#AbstractConfig:base
//#ReferenceLinkers:
//#DriverMode:save-dir-response
//#CompArgs:-fPIC
//#RunEnabled:false
//#LinkerWorkingDirectory:$OUT_DIR

//#AbstractConfig:shared:base
//#LinkArgs:-shared

//#Config:option-looking-value:shared
//#LinkArgs:-rpath -output-name
//#Contains:-output-name

//#Config:existing-directory-value:shared
//#LinkArgs:--soname=. -rpath .
//#ExpectDynamic:DT_SONAME
//#ExpectDynamic:DT_RUNPATH
//#DoesNotContain:.save/

//#Config:response-looking-value:shared
//#Copy:invalid.rsp:invalid.rsp
//#LinkArgs:-rpath @invalid.rsp
//#Contains:@invalid.rsp

//#Config:output-separated:shared
//#PostLinkArgs:--output $OUT_DIR/original-output

//#Config:output-attached:shared
//#PostLinkArgs:--output=$OUT_DIR/original-output

//#Config:nested-response:shared
//#Copy:outer.rsp:outer.rsp
//#Copy:inner.rsp:inner.rsp
//#LinkArgs:@outer.rsp
//#Contains:$ORIGIN/space dir
//#Contains:after-inner

// Replay runs outside the capture working directory, so these relative paths
// must be replaced with paths to the saved copies.
//#Config:retain-symbols:base
//#Copy:retain.txt:retain.txt
//#LinkArgs:-r --retain-symbols-file retain.txt
//#ExpectSym:kept
//#NoSym:dropped

//#Config:export-dynamic-symbol-list:base
//#Copy:symbols.list:symbols.list
//#LinkArgs:-pie --export-dynamic-symbol-list symbols.list
//#ExpectDynSym:kept
//#NoDynSym:dropped

//#Config:dynamic-list:shared
//#Copy:symbols.list:symbols.list
//#LinkArgs:--dynamic-list=symbols.list
//#ExpectDynSym:kept

void kept(void) {}
void dropped(void) {}
void _start(void) {}
