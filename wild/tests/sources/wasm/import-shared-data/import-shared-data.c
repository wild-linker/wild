//#CompArgs: -matomics
//#LinkArgs: --import-memory --export-memory --shared-memory --initial-memory=131072 --max-memory=196608
//#ReferenceLinkers:
//#DiffEnabled:false
//#RunEnabled:false
//#NoSection: Memory
//#ExpectMemoryImport: env/memory initial=2,max=3,shared=true
//#ExpectSection: Start
//#Contains: __wasm_init_memory
//#ExpectSym: memory

int value = 42;

void _start(void) {
  if (value != 42) {
    __builtin_trap();
  }
}
