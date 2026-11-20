//#AbstractConfig:base
//#CompArgs:-fPIC
//#LinkArgs:-shared
//#RunEnabled:false
//#ExpectSym:tls_alias type=tls,section=".tdata",address=0
//#ExpectDynSym:tls_alias type=tls,section=".tdata",address=0
//#ExpectSym:zero_alias type=tls,section=".tbss",address=4
//#ExpectDynSym:zero_alias type=tls,section=".tbss",address=4

//#Config:defsym:base
//#LinkArgs:--defsym=tls_alias=tls_data --defsym=zero_alias=tls_zero

//#Config:script:base
//#AugmentLinkerScript:aliases.ld

__thread int tls_data = 42;
__thread int tls_zero;
