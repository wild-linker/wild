//#AbstractConfig:default
//#LinkerDriver:clang
//#SoSingleLinker:wild
//#Shared:as(libversions.dylib):provider.c
//#LinkSoArgs:-Wl,-install_name,@rpath/libversions.dylib
//#LinkArgs:-Wl,-rpath,@loader_path

//#Config:zero:default
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=0.0.0,compatibility_version=0.0.0

//#Config:versioned:default
//#LinkSoArgs:-Wl,-current_version,3.2.1,-compatibility_version,1.0.0
//#ExpectLoadDylib:@rpath/libversions.dylib current_version=3.2.1
//#ExpectLoadDylib:@rpath/libversions.dylib compatibility_version=1.0.0

//#Config:apple-provider:versioned
//#SoSingleLinker:ld

//#Config:dylib-consumer:versioned
//#LinkArgs:-dynamiclib -Wl,-install_name,@rpath/libconsumer.dylib,-current_version,9.8.7,-compatibility_version,6.5.4
//#ExpectIdDylib:@rpath/libconsumer.dylib current_version=9.8.7,compatibility_version=6.5.4
//#RunDynSym:main

int foo(void);
int main(void) { return foo(); }
