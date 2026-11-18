//#Config:function
//#LinkArgs: --warn-unresolved-symbols
//#RunEnabled: false
//#ExpectWarning: undefined symbol: missing_host_fn
//#ExpectFuncImportCount: 0
//#Contains: undefined_stub

//#Config:allow-then-warn
//#LinkArgs: --allow-undefined --warn-unresolved-symbols
//#RunEnabled: false
//#ExpectWarning: ^$
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:import-function
//#LinkArgs: --import-undefined --warn-unresolved-symbols
//#RunEnabled: false
//#ExpectWarning: ^$
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:import-data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --import-undefined --warn-unresolved-symbols
//#RunEnabled: false
//#ExpectWarning: undefined symbol: missing_data

#ifdef TEST_DATA
extern int missing_data;

void _start(void) { missing_data = 1; }
#else
void missing_host_fn(void);

void _start(void) { missing_host_fn(); }
#endif
