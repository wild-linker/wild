//#Config:z-defs
//#LinkArgs:-Bshareable -z now -z defs
//#Mode:dynamic
//#RunEnabled:false
//#ExpectError:foo
//#CompArgs:-fPIE

//#Config:z-undefs
//#LinkArgs:-Bshareable -z now -z undefs
//#Mode:dynamic
//#RunEnabled:false
//#DiffIgnore:.dynamic.DT_RELA
//#DiffIgnore:.dynamic.DT_RELAENT
//#CompArgs:-fPIE

int foo(void);

int call_foo(void) { return foo() + 2; }
