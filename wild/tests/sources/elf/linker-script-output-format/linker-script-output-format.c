//#AbstractConfig:default
//#RunEnabled:false
//#ReferenceLinkers:lld
//#Arch: x86_64

//#Config:output-format:default
//#LinkerScript:linker-script-output-format.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:with_eb:default
//#LinkArgs:-EB
//#LinkerScript:linker-script-output-format-eb.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:with_el:default
//#LinkArgs:-EL
//#LinkerScript:linker-script-output-format-el.ld
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:unsupported:default
//#ReferenceLinkers:
//#LinkerScript:linker-script-output-format-unsupported.ld
//#ExpectError:elf32-i386 is not yet supported
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:invalid:default
//#ReferenceLinkers:
//#LinkerScript:linker-script-output-format-invalid.ld
//#ExpectError:Setting the output format using OUTPUT_FORMAT is currently unsupported
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

void _start() {}
