//#Config:function
//#LinkArgs: --error-unresolved-symbols
//#ExpectError: undefined symbol: missing_host_fn

//#Config:data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --error-unresolved-symbols
//#ExpectError: undefined symbol: missing_data

//#Config:single-dash
//#LinkArgs: -error-unresolved-symbols
//#ExpectError: undefined symbol: missing_host_fn

//#Config:allow-then-error-function
//#LinkArgs: --allow-undefined --error-unresolved-symbols
//#RunEnabled: false
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:error-then-allow-function
//#LinkArgs: --error-unresolved-symbols --allow-undefined
//#RunEnabled: false
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:allow-then-error-data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --allow-undefined --error-unresolved-symbols
//#RunEnabled: false
//#ExpectFuncImportCount: 0

//#Config:error-then-allow-data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --error-unresolved-symbols --allow-undefined
//#RunEnabled: false
//#ExpectFuncImportCount: 0

//#Config:import-then-error-function
//#LinkArgs: --import-undefined --error-unresolved-symbols
//#RunEnabled: false
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:import-then-error-data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --import-undefined --error-unresolved-symbols
//#ExpectError: undefined symbol: missing_data

//#Config:error-then-import-function
//#LinkArgs: --error-unresolved-symbols --import-undefined
//#RunEnabled: false
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:error-then-import-data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --error-unresolved-symbols --import-undefined
//#ExpectError: undefined symbol: missing_data

#ifdef TEST_DATA
extern int missing_data;

void _start(void) { missing_data = 1; }
#else
void missing_host_fn(void);

void _start(void) { missing_host_fn(); }
#endif
