//#CompArgs: -matomics
//#Object:tls2.c
//#DiffEnabled:false
//#LinkArgs: --export=__tls_base
//#ExpectSym: __tls_base
//#DoesNotContain: env

//#Config:shared-memory:default
//#LinkArgs: --export=__tls_base --shared-memory --max-memory=131072
//#ReferenceLinkers:
//#ExpectErrorWild: shared-memory TLS is not supported yet

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
