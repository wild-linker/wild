// Tests which symbol names can be written without quotes in a version script. As in LLD, unquoted
// names may contain ASCII letters, digits and `_.$/\~=+[]*?-!^`, and a `:` only as part of `::`.
// Anything else needs to be quoted.

//#AbstractConfig:default
//#CompArgs:-fPIC
//#RunEnabled:false
//#LinkArgs:-shared -z now

// Names of only letters, digits and `_`.
//#Config:plain:default
//#LinkArgs:--version-script=./plain.map
//#ExpectDynSym:foo
//#ExpectDynSym:bar
//#NoDynSym:baz

//#Config:namespace:default
//#LinkArgs:--version-script=./namespace.map
//#ExpectDynSym:ns::f
//#ExpectDynSym:ns::gx
//#NoDynSym:baz

// Names like these are common in Rust code and contain `$` and `.`.
//#Config:legacy-rust:default
//#LinkArgs:--version-script=./legacy-rust.map
//#ExpectDynSym:_ZN4core3ptr13drop_in_place$LT$u8$GT$17h0123456789abcdefE
//#ExpectDynSym:_ZN3foo3bar.llvm.123E
//#NoDynSym:baz

//#Config:quoted:default
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--version-script=./quoted.map
//#ExpectDynSym:a:b
//#ExpectDynSym:a/b~c=d+e
//#NoDynSym:baz

// LLD rejects a name that doesn't match any symbol, so only GNU ld is used here.
//#Config:quoted-wasi:default
//#ReferenceLinkers:bfd
//#LinkArgs:--version-script=./quoted-wasi.map
//#ExpectDynSym:foo
//#NoDynSym:baz

// GNU ld doesn't accept `/`, `~`, `=` or `+` in unquoted names, or a leading digit.
//#Config:lld-chars:default
//#ReferenceLinkers:lld
//#LinkArgs:--version-script=./lld-chars.map
//#ExpectDynSym:a/b~c=d+e
//#ExpectDynSym:1abc
//#NoDynSym:baz

//#Config:wasi-name:default
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--version-script=./wasi-name.map
//#ExpectErrorWild:Symbol name "wasi:cli/run@0.2.4#run" contains characters that require quoting

//#Config:single-colon:default
//#ReferenceLinkers:bfd,lld
//#LinkArgs:--version-script=./single-colon.map
//#ExpectErrorWild:Symbol name "a:b" contains characters that require quoting

// GNU ld only warns about non-ASCII bytes, and LLD's error message includes the raw byte, which
// isn't valid UTF-8, so we can only check our own error here.
//#Config:non-ascii:default
//#ReferenceLinkers:
//#LinkArgs:--version-script=./non-ascii.map
//#ExpectErrorWild:Symbol name "café" contains characters that require quoting

int foo = 0;
int bar = 0;
int baz = 0;

// C can't spell these names, so they're defined in assembly.
__asm__(
    ".pushsection .data\n"
    ".globl \"ns::f\", \"ns::gx\", \"a:b\", \"a/b~c=d+e\", \"1abc\"\n"
    ".globl \"_ZN4core3ptr13drop_in_place$LT$u8$GT$17h0123456789abcdefE\"\n"
    ".globl \"_ZN3foo3bar.llvm.123E\"\n"
    "\"ns::f\": .byte 0\n"
    "\"ns::gx\": .byte 0\n"
    "\"a:b\": .byte 0\n"
    "\"a/b~c=d+e\": .byte 0\n"
    "\"1abc\": .byte 0\n"
    "\"_ZN4core3ptr13drop_in_place$LT$u8$GT$17h0123456789abcdefE\": .byte 0\n"
    "\"_ZN3foo3bar.llvm.123E\": .byte 0\n"
    ".popsection\n");
