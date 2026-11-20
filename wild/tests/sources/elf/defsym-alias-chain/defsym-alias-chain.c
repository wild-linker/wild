//#AbstractConfig:base
//#CompArgs:-fPIC
//#LinkArgs:-shared
//#RunEnabled:false

//#AbstractConfig:chain:base
//#ExpectSym:function_alias type=func,section=".text"
//#ExpectDynSym:function_alias type=func,section=".text"
//#ExpectSym:function_alias2 type=func,section=".text"
//#ExpectDynSym:function_alias2 type=func,section=".text"
//#ExpectSym:data_alias type=object,section=".data"
//#ExpectDynSym:data_alias type=object,section=".data"
//#ExpectSym:data_alias2 type=object,section=".data"
//#ExpectDynSym:data_alias2 type=object,section=".data"
//#ExpectSym:tls_alias type=tls,section=".tdata",address=0
//#ExpectDynSym:tls_alias type=tls,section=".tdata",address=0
//#ExpectSym:tls_alias2 type=tls,section=".tdata",address=0
//#ExpectDynSym:tls_alias2 type=tls,section=".tdata",address=0

//#Config:defsym:chain
//#LinkArgs:--defsym=function_alias=function --defsym=function_alias2=function_alias
//#LinkArgs:--defsym=data_alias=data --defsym=data_alias2=data_alias
//#LinkArgs:--defsym=tls_alias=tls_data --defsym=tls_alias2=tls_alias

//#Config:script:chain
//#AugmentLinkerScript:aliases.ld

//#Config:mixed:chain
//#LinkArgs:--defsym=function_alias=function --defsym=data_alias=data
//#LinkArgs:--defsym=tls_alias=tls_data
//#AugmentLinkerScript:chain.ld

//#Config:self:chain
//#AugmentLinkerScript:aliases.ld
//#AugmentLinkerScript:self.ld

//#Config:cycle:base
//#AugmentLinkerScript:cycle.ld
//#ExpectSym:a type=notype,is_abs=true,address=0
//#ExpectDynSym:a type=notype,is_abs=true,address=0
//#ExpectSym:b type=notype,is_abs=true,address=0
//#ExpectDynSym:b type=notype,is_abs=true,address=0

int data = 42;

__thread int tls_data = 42;

int function(void) { return data; }
