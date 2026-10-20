// With shared memory, the main thread's thread-local storage must be set up
// before `_start` runs, as wasm-ld does in `__wasm_init_memory`: `__tls_base`
// points to a block in static memory, initialized from the TLS template.
// (wasi-libc's `wasi_thread_start` does it for other threads; nothing else
// does it for the main thread.)

//#Config:default
//#LinkArgs: --shared-memory --max-memory=131072
//#CompArgs: -matomics -mbulk-memory
//#ExpectSharedMemory: true

//#Config:stack-first
//#LinkArgs: --shared-memory --max-memory=131072 --stack-first
//#CompArgs: -matomics -mbulk-memory
//#ExpectSharedMemory: true

extern char __heap_base;

_Thread_local int counter = 42;
_Thread_local int more[4] = {1, 2, 3, 4};
_Thread_local char zeroed[16];
int global_after = 7;

void _start(void) {
  char* tls = __builtin_wasm_tls_base();
  if (tls == 0) {
    __builtin_trap();
  }
  // Static memory, below the heap.
  if (tls + __builtin_wasm_tls_size() > &__heap_base) {
    __builtin_trap();
  }
  if (counter != 42 || more[0] != 1 || more[3] != 4) {
    __builtin_trap();
  }
  for (int i = 0; i < 16; i++) {
    if (zeroed[i] != 0) {
      __builtin_trap();
    }
  }
  counter += 1;
  more[2] = 30;
  zeroed[5] = 9;
  // The block doesn't overlap ordinary data.
  if (counter != 43 || more[2] != 30 || zeroed[5] != 9 || global_after != 7) {
    __builtin_trap();
  }
}
