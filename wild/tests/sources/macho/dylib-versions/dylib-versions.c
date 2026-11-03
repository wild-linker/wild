//#AbstractConfig:default
//#LinkerDriver:clang
//#LinkArgs:-dynamiclib -Wl,-install_name,@rpath/libversions.dylib
//#RunDynSym:foo
//#ExpectIdDylib:@rpath/libversions.dylib current_version=0.0.0,compatibility_version=0.0.0

//#Config:defaults:default

//#Config:current-only:default
//#LinkArgs:-Wl,-current_version,3.2.1
//#ExpectIdDylib:@rpath/libversions.dylib current_version=3.2.1

//#Config:compatibility-only:default
//#LinkArgs:-Wl,-compatibility_version,2.1.3
//#ExpectIdDylib:@rpath/libversions.dylib compatibility_version=2.1.3

//#Config:both:default
//#LinkArgs:-Wl,-current_version,3.2.1,-compatibility_version,1.0.0
//#ExpectIdDylib:@rpath/libversions.dylib current_version=3.2.1,compatibility_version=1.0.0

//#Config:abbreviated:default
//#LinkArgs:-Wl,-current_version,3.2,-compatibility_version,1
//#ExpectIdDylib:@rpath/libversions.dylib current_version=3.2.0,compatibility_version=1.0.0

//#Config:maximum:default
//#LinkArgs:-Wl,-current_version,65535.255.255,-compatibility_version,65535.255.255
//#ExpectIdDylib:@rpath/libversions.dylib current_version=65535.255.255,compatibility_version=65535.255.255

//#Config:repeated:default
//#LinkArgs:-Wl,-current_version,9,-compatibility_version,8
//#LinkArgs:-Wl,-current_version,3.2.1,-compatibility_version,1.0.0
//#ExpectIdDylib:@rpath/libversions.dylib current_version=3.2.1,compatibility_version=1.0.0

//#Config:compatibility-greater:default
//#LinkArgs:-Wl,-current_version,1,-compatibility_version,2
//#ExpectIdDylib:@rpath/libversions.dylib current_version=1.0.0,compatibility_version=2.0.0

int foo(void) { return 42; }
