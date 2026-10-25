//#AbstractConfig:default

//#Config:strip-all:default
//#Object:runtime.c
//#LinkArgs:--strip-all
//#ReferenceLinkers:bfd,lld
//#DiffIgnore:file-header.entry
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:single-threaded:default
//#Object:runtime.c
//#WildExtraLinkArgs:--threads=1
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:dev_null:default
//#Object:runtime.c
//#LinkArgs:-o /dev/null
//#DiffEnabled:false
//#RunEnabled:false
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:gc-sections:default
//#CompArgs:-g -ffunction-sections
//#LinkArgs:--gc-sections
//#Object:runtime.c
//#NoSym:this_function_is_not_used
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-args
//#AutoAddObjects:false
//#ExpectError:no input files
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-mmap-output
//#Object:runtime.c
//#ReferenceLinkers:lld
//#LinkArgs:--no-mmap-output-file
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:fallocate-output
//#Object:runtime.c
//#ReferenceLinkers:
//#LinkArgs:--fallocate-output-file
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:madvise-huge-pages
//#RequiresTransparentHugePages:true
//#Object:runtime.c
//#ReferenceLinkers:
//#LinkArgs:--madvise-huge-pages --mmap-output-file
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:madvise-huge-pages-no-mmap
//#Object:runtime.c
//#ReferenceLinkers:
//#LinkArgs:--madvise-huge-pages --no-mmap-output-file
//#ExpectErrorWild:--madvise-huge-pages requires mmapped output file
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:disable-output-allocation-options
//#Object:runtime.c
//#ReferenceLinkers:
//#LinkArgs:--no-fallocate-output-file --no-madvise-huge-pages
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

// The later --strip-all flag should override --strip-debug.
//#Config:strip-debug-strip-all
//#Object:runtime.c
//#LinkArgs:--strip-debug --strip-all
//#DiffIgnore:file-header.entry
//#NoSym:_start
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

// The later --strip-debug flag should override --strip-all.
//#Config:strip-all-strip-debug
//#Object:runtime.c
//#LinkArgs:--strip-all --strip-debug
//#ExpectSym:_start
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:retain-symbols-file
//#Object:runtime.c
//#LinkArgs:--retain-symbols-file ./link_args.retain
//#ExpectSym:_start
//#ExpectSym:exit_syscall
//#NoSym:runtime_init
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:sym-info
//#ReferenceLinkers:
//#Object:runtime.c
//#LinkArgs:--sym-info=this_function_is_not_used
//#ExpectMessage:Global name `this_function_is_not_used` refers to: Some\(sym-[0-9]+\)
//#ExpectMessage:Matching/related names
//#ExpectMessage:[0-9]+: Global Func: NON_INTERPOSABLE | DIRECT
//#ExpectMessage:this_function_is_not_used
//#ExpectMessage:#[0-9]+ in File #256 \(1/0\) .* \(LOADED\)
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:write-gc-stats:default
//#ReferenceLinkers:
//#CompArgs:-ffunction-sections
//#Object:runtime.c
//#LinkArgs:--gc-sections --write-gc-stats=$OUT_DIR/gc-stats.txt
//#NoSym:this_function_is_not_used
//#AssertOutputFileMatches:gc-stats.txt:Discarded .* of executable code
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:unsupported-z-flag
//#Object:runtime.c
//#LinkArgs:-z foobar
//#ExpectWarning:warning.*foobar
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}

void this_function_is_not_used(void) {}
