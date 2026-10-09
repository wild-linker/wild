//#Config:default
//#LinkArgs: --import-memory
//#NoSection: Memory
//#ExpectSection: Import
//#ExpectMemoryImport: env/memory
//#NoSym: memory
//#RunEnabled: false

//#Config:shared
//#CompArgs: -matomics -DSHARED_MEMORY_CHECK
//#LinkArgs: --import-memory --shared-memory --initial-memory=131072 --max-memory=196608
//#WasmPreload: env=provider.wat
//#DiffEnabled:false
//#NoSection: Memory
//#ExpectMemoryImport: env/memory initial=2,max=3,shared=true
//#ExpectSection: Start
//#Contains: __wasm_init_memory
//#NoSym: memory

//#Config:shared-exported
//#CompArgs: -matomics
//#LinkArgs: --import-memory --export-memory --shared-memory --initial-memory=131072 --max-memory=196608
//#ReferenceLinkers:
//#DiffEnabled:false
//#RunEnabled:false
//#NoSection: Memory
//#ExpectMemoryImport: env/memory initial=2,max=3,shared=true
//#ExpectSym: memory

//#Config:import-max
//#LinkArgs: --import-memory --initial-memory=131072 --max-memory=196608 -z stack-size=65536 --stack-first
//#NoSection: Memory
//#ExpectMemoryImport: env/memory initial=2,max=3,shared=false
//#NoSym: memory
//#RunEnabled: false

//#Config:exported
//#LinkArgs: --import-memory --export-memory
//#RunEnabled: false
//#NoSection: Memory
//#ExpectMemoryImport: env/memory
//#ExpectSection: Export
//#ExpectSym: memory

//#Config:exported-values
//#LinkArgs: --import-memory=foo,bar --export-memory
//#RunEnabled: false
//#NoSection: Memory
//#ExpectMemoryImport: foo/bar
//#ExpectSection: Export
//#ExpectSym: memory

//#Config:input-memory-export
//#Object:mem-export.wat
//#LinkArgs: --import-memory
//#RunEnabled: false
//#NoSection: Memory
//#ExpectMemoryImport: env/memory
//#NoSym: from_input

//#Config:import-name-only
//#LinkArgs: --import-memory=mymem
//#RunEnabled: false
//#NoSection: Memory
//#ExpectMemoryImport: env/mymem
//#NoSym: memory
// A comma-less value became the name in llvm/llvm-project#160409, released in LLVM 22. Older
// wasm-ld parses it as the module with an empty name.
// TODO(wasm): Drop ReferenceLinkers once CI's wasm-ld is LLVM 22 or newer.
//#ReferenceLinkers:

#ifdef SHARED_MEMORY_CHECK
__attribute__((import_module("env"), import_name("read_at"))) extern int read_at(int addr);
__attribute__((import_module("env"), import_name("write_at"))) extern void write_at(int addr,
                                                                                    int value);

volatile int value = 7;

void _start(void) {
  if (value != 7) {
    __builtin_trap();
  }
  write_at((int)&value, 5);
  if (value != 5) {
    __builtin_trap();
  }
  value = 9;
  if (read_at((int)&value) != 9) {
    __builtin_trap();
  }
}
#else
void _start(void) {}
#endif
