//#AbstractConfig:base
//#Arch: x86_64
//#LinkArgs:-r
//#RunEnabled:false
//#TestUpdateInPlace:true
//#ExpectSection:.text.group_member flags=AXG
//#ExpectSection:.group type=17

//#Config:default:base
//#Config:strip-signature:base
//#LinkArgs:-r --retain-symbols-file=./keep.txt

.section .text.group_member,"axG",@progbits,group_signature,comdat
.globl group_signature
.type group_signature,@function
group_signature:
  ret
.size group_signature, .-group_signature
