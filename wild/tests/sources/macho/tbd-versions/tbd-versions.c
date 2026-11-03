//#AbstractConfig:default
//#LinkerDriver:clang
//#LinkerWorkingDirectory:$OUT_DIR
//#LinkArgs:-L. -lversions
//#RunEnabled:false

//#Config:defaults:default
//#Copy:defaults.tbd:libversions.tbd
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=1.0.0,compatibility_version=1.0.0

//#Config:current-only:default
//#Copy:current-only.tbd:libversions.tbd
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=3.2.1,compatibility_version=1.0.0

//#Config:compatibility-only:default
//#Copy:compatibility-only.tbd:libversions.tbd
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=1.0.0,compatibility_version=2.1.0

//#Config:zero:default
//#Copy:zero.tbd:libversions.tbd
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=0.0.0,compatibility_version=0.0.0

//#Config:reexport:default
//#Copy:reexport.tbd:libversions.tbd
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=7.8.9,compatibility_version=2.3.4

int foo(void);
int main(void) { return foo(); }
