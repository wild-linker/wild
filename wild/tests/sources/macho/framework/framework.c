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

//#Config:framework-relative-sysroot:custom-sdk
//#LinkerWorkingDirectory:$OUT_DIR
//#Object:template(-Wl,-syslibroot,sdk -F/Custom/Frameworks -framework Foo $O):main.c

//#Config:save-framework-relative-sysroot:framework-relative-sysroot
//#DriverMode:save-dir-response

//#AbstractConfig:search:default
//#LinkArgs:-nostdlib -dynamiclib
//#LinkSoArgs:-lSystem
//#RunEnabled:false
//#Shared:as(host/Foo.framework/Foo),noadd:foo.c
// Export a different symbol so a successful link identifies the chosen framework.
//#Shared:as(alternate/Foo.framework/Foo),noadd:alternate.c
//#Copy:foo.c:sdk/.keep

//#Config:sdk-directory-missing:search
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F$OUT_DIR/host -framework Foo $O):main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#Config:sdk-path-is-file:search
//#Copy:foo.c:sdk/$OUT_DIR/host
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F$OUT_DIR/host -framework Foo $O):main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

// The SDK directory exists but contains no Foo. It must hide the host Foo.
//#Config:sdk-directory-hides-host:search
//#Copy:foo.c:sdk/$OUT_DIR/host/.keep
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F$OUT_DIR/host -framework Foo $O):main.c
//#ExpectError:framework.*Foo

//#Config:sdk-directory-preferred:search
//#Copy:$OUT_DIR/alternate/Foo.framework/Foo:sdk/$OUT_DIR/host/Foo.framework/Foo
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F$OUT_DIR/host -framework Foo $O):alternate-main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

// A missing framework in the chosen SDK directory advances to the next search
// directory, rather than falling back to the corresponding host directory.
//#Config:sdk-search-continues:search
//#Copy:foo.c:sdk/$OUT_DIR/host/.keep
//#Copy:$OUT_DIR/alternate/Foo.framework/Foo:sdk/Next/Foo.framework/Foo
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F$OUT_DIR/host -F/Next -framework Foo $O):alternate-main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

// Relative -F paths are never prefixed with the sysroot.
//#Config:relative-framework-path:search
//#Copy:$OUT_DIR/alternate/Foo.framework/Foo:sdk/host/Foo.framework/Foo
//#LinkerWorkingDirectory:$OUT_DIR
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -Fhost -framework Foo $O):main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#Config:sdk-default-library-frameworks:search
//#Copy:$OUT_DIR/alternate/Foo.framework/Foo:sdk/Library/Frameworks/Foo.framework/Foo
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -framework Foo $O):alternate-main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#Config:sdk-default-system-frameworks:search
//#Copy:$OUT_DIR/alternate/Foo.framework/Foo:sdk/System/Library/Frameworks/Foo.framework/Foo
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -framework Foo $O):alternate-main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#AbstractConfig:search-order:search
//#Copy:$OUT_DIR/host/Foo.framework/Foo:sdk/First/Foo.framework/Foo
//#Copy:$OUT_DIR/alternate/Foo.framework/Foo:sdk/Last/Foo.framework/Foo

//#Config:sdk-search-order:search-order
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F/First -F/Last -framework Foo $O):main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo

//#Config:sdk-search-order-reversed:search-order
//#Object:template(-Wl,-syslibroot,$OUT_DIR/sdk -F/Last -F/First -framework Foo $O):alternate-main.c
//#ExpectLoadDylib:@rpath/Foo.framework/Foo
