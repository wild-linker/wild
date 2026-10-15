//#AbstractConfig:base
//#Arch:x86_64
//#Object:strong.s
//#LinkArgs:-r
//#RunEnabled:false
//#ReferenceLinkers:bfd,lld
//#ExpectSym:signature binding=global,section=".data"
//#ExpectSym:.Lsignature binding=local,section=".group"

//#Config:default:base

//#Config:retain:base
// Some versions of LLD - e.g. 23.1.1 seem to do things differently here.
//#ReferenceLinkers:bfd
//#LinkArgs:--retain-symbols-file=./keep.txt

//#Config:discard-locals:base
//#ReferenceLinkers:bfd
//#LinkArgs:-x

//#Config:collected-signature:base
//#ReferenceLinkers:
//#LinkArgs:--gc-sections -u member
//#ExpectErrorWild:Missing group signature symbol

.section .data.member,"awG",@progbits,signature,comdat
.weak signature
.globl member
member:
signature:
  .long 17

.section .data.local,"awG",@progbits,.Lsignature,comdat
  .long 25

.section .note.GNU-stack,"",@progbits
