//#Config:default
// LLD uses a zero entry point instead of falling back to the start of .text.
//#ReferenceLinkers:bfd
//#LinkArgs:--entry=missing --no-gc-sections
//#ExpectWarning:cannot find entry symbol .*missing
//#ExpectEntry:first_function
//#RunEnabled:false
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

void first_function(void) {}
