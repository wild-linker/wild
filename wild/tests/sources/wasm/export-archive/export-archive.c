//#AbstractConfig:base
//#Archive:export-archive-other.c,export-archive-member.c

//#Config:export:base
//#LinkArgs:--no-entry --export=exported_from_archive --export=archived_data --export-if-defined=defined_here
//#WasmRunner:driver.wat
//#ExpectSym:exported_from_archive
//#ExpectSym:archived_data
//#ExpectSym:defined_here
//#ExpectSym:memory
//#NoSym:_start
//#Contains:export_member
//#DoesNotContain:other_member

//#Config:if-defined:base
//#LinkArgs:--export-if-defined=exported_from_archive --export-if-defined=archived_data --export-if-defined=defined_here --export-if-defined=does_not_exist
//#ExpectSym:exported_from_archive
//#ExpectSym:archived_data
//#ExpectSym:defined_here
//#ExpectSym:_start
//#ExpectSym:memory
//#NoSym:does_not_exist
//#NoSym:other_archived
//#Contains:export_member
//#DoesNotContain:other_member

//#Config:missing:base
//#LinkArgs:--export=does_not_exist
//#ExpectError:symbol exported via --export not found: does_not_exist

int defined_here(void) { return 1; }

void _start(void) {}
