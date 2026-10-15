//#AbstractConfig:base
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld

//#Config:partial:base
//#Object:reference.s
//#Object:winner.s
//#Object:loser.s
//#LinkArgs:-r
//#RunEnabled:false
# linker-diff cannot compare code at relocatable address zero. The resolved config checks it.
//#DiffIgnore:section-diff-failed..text
//#ExpectSym:missing binding=global,type=func

//#Config:weak:base
//#Object:reference.s
//#Object:winner.s
//#Object:loser.s
//#CompArgs:-Wa,--defsym,WEAK=1
//#LinkArgs:-r
//#RunEnabled:false
//#DiffIgnore:section-diff-failed..text
//#ExpectSym:missing binding=weak,type=func

//#Config:resolved:base
//#Relocatable:reference.s,winner.s,loser.s
//#Object:definition.s

//#Config:unresolved:base
//#Relocatable:reference.s,winner.s,loser.s
//#ExpectError:missing

.section .note.GNU-stack,"",@progbits
