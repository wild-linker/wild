// With shared memory, a program whose only data is thread-local still gets a
// `__wasm_init_memory`, which sets up the main thread's TLS.

//#Config:default
//#LinkArgs: --shared-memory --max-memory=131072
//#CompArgs: -matomics -mbulk-memory
//#ExpectSharedMemory: true

//#Config:stack-first
//#LinkArgs: --shared-memory --max-memory=131072 --stack-first
//#CompArgs: -matomics -mbulk-memory
//#ExpectSharedMemory: true

_Thread_local int counter = 42;
_Thread_local long long wide[2] = {1, 2};

void _start(void) {
  if (__builtin_wasm_tls_base() == 0) {
    __builtin_trap();
  }
  if (counter != 42 || wide[0] != 1 || wide[1] != 2) {
    __builtin_trap();
  }
  if (((unsigned long)&wide[0] & 7) != 0) {
    __builtin_trap();
  }
  counter += 1;
  if (counter != 43) {
    __builtin_trap();
  }
}
