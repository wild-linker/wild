//#CompArgs: -matomics
//#Object:tls2.c
//#DiffEnabled:false
//#LinkArgs: --export=__tls_base
//#ExpectSym: __tls_base
//#DoesNotContain: env

//#Config:shared-memory:default
//#LinkArgs: --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#RunEnabled:false
//#ExpectSym: __tls_base address=0
//#ExpectSym: __tls_size
//#ExpectSym: __tls_align
//#ExpectSym: __wasm_init_tls
//#ExpectSection: DataCount
//#ExpectSharedMemory: true

_Thread_local int tls1 = 1;
extern _Thread_local int tls2;
_Thread_local int tls_bss;

void _start(void) {
  if (tls1 != 1) {
    __builtin_trap();
  }
  if (tls2 != 2) {
    __builtin_trap();
  }
  if (tls_bss != 0) {
    __builtin_trap();
  }

  tls1 = 10;
  tls2 = 20;
  tls_bss = 30;

  if (tls1 != 10 || tls2 != 20 || tls_bss != 30) {
    __builtin_trap();
  }
}
