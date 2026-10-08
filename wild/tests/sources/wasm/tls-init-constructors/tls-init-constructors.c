//#CompArgs: -matomics -mbulk-memory
//#LinkArgs: --shared-memory --max-memory=131072 --no-entry --export=check --export=__heap_base
//#ReferenceLinkers:
//#DiffEnabled:false
//#WasmRunner: driver.wat
//#ExpectSym: __wasm_init_tls
//#ExpectSym: check

// The host must initialize TLS before calling an export that runs constructors.
_Thread_local int tls_value = 42;
static int constructor_calls;

__attribute__((constructor)) static void init(void) {
  if (tls_value != 42) {
    __builtin_trap();
  }
  ++constructor_calls;
}

void check(void) {
  if (tls_value != 42 || constructor_calls != 1) {
    __builtin_trap();
  }
}
