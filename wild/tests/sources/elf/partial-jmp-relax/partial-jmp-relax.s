//#AbstractConfig:default
//#Arch:x86_64
//#DiffEnabled:false

//#Config:partial:default
//#Object:jump.s
//#LinkArgs:-r
//#RunEnabled:false
//#ExpectSectionBytes:.text=0xff2500000000 0..6
//#ExpectReloc:.text offset=2,target=target,type=41,addend=-4

//#Config:final:default
//#Relocatable:jump.s
//#LinkArgs:-static

.section .note.GNU-stack,"",@progbits
