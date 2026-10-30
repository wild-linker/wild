//#AbstractConfig:default
//#CompArgs:-fPIC
//#ReferenceLinkers:bfd,lld
//#TestOutputToDevNull:true
//#RunEnabled:false

// File writing goes through slightly different code paths with 1 vs more than 1 threads.
//#Config:executable-single-thread:default
//#LinkArgs:-static
//#WildExtraLinkArgs:--threads=1

//#Config:executable-multi-thread:default
//#LinkArgs:-static
//#WildExtraLinkArgs:--threads=2

//#Config:shared-single-thread:default
//#LinkArgs:-shared
//#WildExtraLinkArgs:--threads=1

//#Config:shared-multi-thread:default
//#LinkArgs:-shared
//#WildExtraLinkArgs:--threads=2

void _start(void) {}
