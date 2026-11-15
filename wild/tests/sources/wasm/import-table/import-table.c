//#Config:default
//#LinkArgs:--import-table
//#WasmPreload:env=import-table2.wat
//#NoSection:Table
//#ExpectSection:Element
//#ExpectTableImport:env/__indirect_function_table initial=3,max=none
//#NoSym:__indirect_function_table
//#ExpectSym:_start

//#Config:growable
// `--growable-table` removes the maximum only from a defined table. An import already has none.
//#LinkArgs:--import-table --growable-table
//#WasmPreload:env=import-table2.wat
//#NoSection:Table
//#ExpectSection:Element
//#ExpectTableImport:env/__indirect_function_table initial=3,max=none
//#NoSym:__indirect_function_table

//#Config:table-base
//#LinkArgs:--import-table --table-base=100
//#WasmPreload:env=import-table2.wat
//#NoSection:Table
//#ExpectSection:Element
//#ExpectTableImport:env/__indirect_function_table initial=102,max=none
//#NoSym:__indirect_function_table

//#Config:unused
// No address-taken function, so the table is not created.
//#CompArgs:-DNO_INDIRECT
//#LinkArgs:--import-table
//#NoSection:Table
//#NoSection:Element
//#NoSection:Import
//#NoSym:__indirect_function_table

//#Config:export-table
//#LinkArgs:--import-table --export-table
//#ExpectError:--import-table and --export-table may not be used together

#ifndef NO_INDIRECT
static int inc(int x) { return x + 1; }

static int dec(int x) { return x - 1; }

typedef int (*Fn)(int);
#endif

void _start(void) {
#ifndef NO_INDIRECT
  Fn a = inc;
  Fn b = dec;
  if (a(1) != 2 || b(3) != 2) {
    __builtin_trap();
  }
#endif
}
