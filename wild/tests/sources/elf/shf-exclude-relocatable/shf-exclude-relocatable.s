/* 
//#AbstractConfig:common
//#RunEnabled:false

//#Config:partial:common
//#Object:shf-exclude-relocatable-keep.s
//#Object:shf-exclude-relocatable-drop.s
//#Object:shf-exclude-relocatable-ref.s
//#LinkArgs:-r
//#DiffEnabled:false
//#ExpectSym:excl_var section=.data.excl,type=object,size=4
//#ExpectSym:keep_fn section=.text.startup,type=func
//#ExpectSym:drop_fn section=.text.startup,type=func
//#ExpectSym:use_excl section=.text,type=func
//#ExpectSection:.data.excl flags=WAE
//#ExpectSection:.exclude flags=E
//#ExpectSection:.rela.text
//#ExpectSectionBytes:.data.excl=0x25000000
//#Contains:ShouldBeExcluded

//#Config:final:common
//#ReferenceLinkers:bfd
//#Relocatable:shf-exclude-relocatable-keep.s,shf-exclude-relocatable-drop.s
//#LinkArgs:-no-pie
//#ExpectSym:_start section=.text,type=func
//#ExpectSym:keep_fn section=.text,type=func
//#NoSection:.text.startup
//#NoSym:drop_fn
//#NoSym:excl_var
//#NoSym:use_excl
//#NoSection:.data.excl
//#NoSection:.exclude
//#DoesNotContain:ShouldBeExcluded
*/
.section .text, "ax", @progbits
.globl _start
.type _start, @function
_start:
  .dc.a keep_fn
  .byte 0
.size _start, .-_start

.section .note.GNU-stack, "", @progbits
