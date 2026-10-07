//#AbstractConfig:default
//#LinkerDriver:clang
//#SoSingleLinker:wild
//#LinkSoArgs:-Wl,-install_name,@rpath/Foo.framework/Foo

//#Config:framework:default
//#Object:template(-F$OUT_DIR -framework Foo -Wl,-rpath,@loader_path $O):main.c
//#Shared:as(Foo.framework/Foo),noadd:foo.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#Config:sdk-tbd:default
//#Object:template(-framework CoreFoundation $O):core-foundation.c
//#ExpectLoadDylib:/System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation

//#AbstractConfig:custom-sdk:default
//#ReferenceLinkers:ld
// The final dylib only links our framework, so the temporary SDK needs no system libraries.
//#LinkArgs:-nostdlib -dynamiclib
//#LinkSoArgs:-lSystem
//#RunEnabled:false
//#Shared:as(sdk/Custom/Frameworks/Foo.framework/Foo),noadd:foo.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#Config:framework-before-sysroot:custom-sdk
//#Object:template(-F/Custom/Frameworks -Wl,-syslibroot,$OUT_DIR/sdk -framework Foo $O):main.c

//#Config:framework-after-sysroot:custom-sdk
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F /Custom/Frameworks -framework Foo $O):main.c

//#Config:save-framework-before-sysroot:framework-before-sysroot
//#DriverMode:save-dir-response

//#Config:save-framework-after-sysroot:framework-after-sysroot
//#DriverMode:save-dir-response
