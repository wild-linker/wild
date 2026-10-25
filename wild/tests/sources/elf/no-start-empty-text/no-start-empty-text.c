//#Config:default
//#SkipArch:ppc64le
//#RunEnabled:false
//#ExpectWarning:cannot find entry symbol
//#ExpectEntry:0
//#DiffIgnore:file-header.*
//#DiffIgnore:riscv_attributes.stack_align
//#DiffIgnore:segment.RISCV_ATTRIBUTES.*
//#DiffIgnore:riscv_attributes.*
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

int data = 42;
