//#Config:function
//#LinkArgs: --import-undefined
//#RunEnabled: false
//#ExpectFuncImport: env/missing_host_fn=1
//#ExpectFuncImportCount: 1

//#Config:data
//#CompArgs: -DTEST_DATA
//#LinkArgs: --import-undefined
//#ExpectError: undefined symbol: missing_data

#ifdef TEST_DATA
extern int missing_data;

void _start(void) { missing_data = 1; }
#else
void missing_host_fn(void);

void _start(void) { missing_host_fn(); }
#endif
