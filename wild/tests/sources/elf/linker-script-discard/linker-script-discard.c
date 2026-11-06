//#AbstractConfig:base
//#Mode:dynamic
//#RunEnabled:false
//#ReferenceLinkers:lld
//#DiffIgnore:section.got
//#DiffIgnore:segment.LOAD.RX.alignment
//#DiffIgnore:segment.LOAD.RWX.alignment
// Wild does not emit the `.eh_frame` section as all code sections are discarded, but lld still
// emits the CIE.
//#DiffIgnore:section.eh_frame
//#DoesNotContain:/DISCARD/
//#CompArgs:-fPIC
//#NoSym:discard_var_1
//#NoSym:discard_var_2

//#Config:default:base
//#LinkArgs:-shared -z now -T ./linker-script-discard.ld
//#DoesNotContain:.text
//#NoSym:foo

//#Config:multiple-discard:base
//#LinkArgs:-shared -z now -T ./multiple-discard.ld
//#ExpectSym:foo
//#ExpectSym:kept_data
//#ExpectProgramHeader:LOAD flags=RX,sections=[.text,*]
//#ExpectProgramHeader:LOAD flags=RW,sections=[.data,*]

__attribute__((section(".discard_before"))) int discard_var_1 = 1;
__attribute__((section(".discard_after"))) int discard_var_2 = 2;
int kept_data = 42;

int foo() { return kept_data; }
