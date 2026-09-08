//#Arch:x86_64
//#LinkArgs:-r --gc-sections -u retained
//#RunEnabled:false
//#ExpectSection:.group type=17
//#ExpectSection:.data.retained
//#ExpectSection:.data.companion flags=WAG
//#NoSection:.data.unused
//#ExpectSym:retained
//#ExpectSym:companion

.section .data.retained,"awGR",@progbits,signature,comdat
.globl retained
retained:
  .long 42

.section .data.companion,"awG",@progbits,signature,comdat
.globl companion
companion:
  .long 17

.section .data.unused,"awG",@progbits,unused_signature,comdat
.long 99
.section .note.GNU-stack,"",@progbits
