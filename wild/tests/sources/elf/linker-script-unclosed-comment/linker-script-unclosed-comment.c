//#RunEnabled:false
//#ReferenceLinkers:
//#LinkArgs:-T ./linker-script-unclosed-comment.ld
//#ExpectError:Unclosed comment
//#CompArgs:-fPIE
//#LinkArgs:-no-pie
void _start() {}
