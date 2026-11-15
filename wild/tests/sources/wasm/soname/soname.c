//#Config:default
//#ExpectSection:name
//#ExpectModuleName:$OUTPUT

//#Config:equals
//#LinkArgs:--soname=libfoo.so
//#ExpectModuleName:libfoo.so

//#Config:separate
//#LinkArgs:--soname libbar.so
//#ExpectModuleName:libbar.so

//#Config:single-dash
//#LinkArgs:-soname=libbaz.so
//#ExpectModuleName:libbaz.so

//#Config:last-wins
//#LinkArgs:--soname=first.so --soname=second.so
//#ExpectModuleName:second.so

//#Config:path
//#LinkArgs:--soname=dir/libqux.so
//#ExpectModuleName:dir/libqux.so

//#Config:empty
//#LinkArgs:--soname=
//#ExpectModuleName:$OUTPUT

//#Config:strip-all
//#LinkArgs:--strip-all --soname=libstripped.so
//#NoSection:name
//#DoesNotContain:libstripped.so

//#Config:keep-name
//#LinkArgs:--strip-all --keep-section=name --soname=libkept.so
//#ExpectSection:name
//#ExpectModuleName:libkept.so

void _start(void) {}
