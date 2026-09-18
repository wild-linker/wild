//#Config:default
//#LinkArgs: --import-memory
//#NoSection: Memory
//#ExpectSection: Import
//#ExpectMemoryImport: env/memory
//#NoSym: memory
//#RunEnabled: false

//#Config:shared
//#CompArgs: -matomics
//#LinkArgs: --import-memory --shared-memory --initial-memory=131072 --max-memory=196608
// wasm-ld accepts this combination; Wild doesn't support it yet, so only run Wild.
//#ReferenceLinkers:
//#ExpectError: --import-memory with --shared-memory is not yet supported

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

void _start(void) {}
