//#AbstractConfig:default
//#Arch:x86_64
//#LinkerDriver:gcc
//#ReferenceLinkers:bfd,lld
//#DiffIgnore:section.rodata
//#CompArgs:-ftls-model=local-exec
//#Object:local-dynamic.s
//#ExpectProgramHeader:TLS mem-size=9,sections=[.tdata,.tbss]

//#Config:pie:default
//#LinkArgs:-pie
//#ExpectSym:_TLS_MODULE_BASE_ type=tls,binding=local

//#Config:non-pie:default
//#LinkArgs:-no-pie
//#DiffIgnore:section.data
//#ExpectSym:_TLS_MODULE_BASE_ type=tls,binding=local
// LLD relaxes to lea. Wild and BFD relax to mov.
//#DiffMatchAny:true

//#Config:linker-script:default
//#LinkerDriver:none
//#LinkerScript:module-base.ld
//#RunEnabled:false
//#ExpectSym:_TLS_MODULE_BASE_ type=tls,binding=local
//#ExpectSym:script_start address=0x10000

_Thread_local long initialized = 1;
_Thread_local char value;

char* local_dynamic_address(void);

int main(void) {
  if (local_dynamic_address() != &value) {
    return 1;
  }
  if (initialized != 1) {
    return 2;
  }
  return 42;
}
