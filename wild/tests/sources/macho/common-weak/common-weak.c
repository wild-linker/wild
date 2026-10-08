//#AbstractConfig:default
//#LinkerDriver:clang
//#ReferenceLinkers:ld,lld
//#ExpectDynSym:_value binding=weak

//#Config:common-first:default
//#Object:common.c:-fcommon
//#Object:weak.c

//#Config:weak-first:default
//#Object:weak.c
//#Object:common.c:-fcommon

extern int value;

int main(void) { return value; }
