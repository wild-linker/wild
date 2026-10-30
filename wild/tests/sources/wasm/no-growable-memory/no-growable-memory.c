//#Config:default
//#LinkArgs: --no-growable-memory -z stack-size=65536 --stack-first

//#Config:initial-memory
//#LinkArgs: --initial-memory=131072 --no-growable-memory -z stack-size=65536 --stack-first
//#CompArgs: -DEXPECTED_HEAP_END=131072UL

//#Config:initial-heap
//#LinkArgs: --initial-heap=65536 --no-growable-memory -z stack-size=65536 --stack-first
//#CompArgs: -DCHECK_HEAP_END

//#Config:max-memory-zero
//#LinkArgs: --max-memory=0 --no-growable-memory -z stack-size=65536 --stack-first

//#Config:incompatible
//#LinkArgs: --max-memory=131072 --no-growable-memory -z stack-size=65536 --stack-first
//#ExpectError: --max-memory is incompatible with --no-growable-memory

extern char __heap_base;
extern char __heap_end;

void _start(void) {
#ifdef EXPECTED_HEAP_END
  if ((unsigned long)&__heap_end != EXPECTED_HEAP_END) {
    __builtin_trap();
  }
#endif
#ifdef CHECK_HEAP_END
  unsigned long expected = ((unsigned long)&__heap_base + 65536UL + 65535UL) & ~65535UL;
  if ((unsigned long)&__heap_end != expected) {
    __builtin_trap();
  }
#endif
  if (__builtin_wasm_memory_grow(0, 1) != (unsigned long)-1) {
    __builtin_trap();
  }
}
