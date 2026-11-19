//#CompArgs:-fPIC -fcf-protection=none
//#LinkArgs:-shared --build-id --defsym=header_alias=__ehdr_start
//#RunEnabled:false
// BFD does not resolve __ehdr_start in a --defsym expression.
//#ReferenceLinkers:lld
//#ExpectSym:header_alias is_abs=false,type=notype,section=".note.gnu.build-id"
//#ExpectDynSym:header_alias is_abs=false,type=notype,section=".note.gnu.build-id"

void function(void) {}
