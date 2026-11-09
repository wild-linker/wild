.section __TEXT,__cstring,cstring_literals
.p2align 0

.globl _s2w
_s2w:
  .asciz "World"

.globl _s2h
_s2h:
  .asciz "Hello"

// Reference a local label in its own cstring section.
.section __TEXT,__literal_loc1,cstring_literals
.p2align 0
Lloc1:
  .asciz "Local1"

.section __TEXT,__custom_cstring,cstring_literals
.p2align 0

.globl _s4h
_s4h:
  .asciz "Hello"

.section __TEXT,__text,regular,pure_instructions
.p2align 2

.globl _get_loc1
_get_loc1:
  adrp x0, Lloc1@PAGE
  add x0, x0, Lloc1@PAGEOFF
  ret

// Exercise GOT references to merged symbols from another object and this object.
.globl _get_s1w
_get_s1w:
  adrp x0, _s1w@GOTPAGE
  ldr x0, [x0, _s1w@GOTPAGEOFF]
  ret

.globl _get_s2w
_get_s2w:
  adrp x0, _s2w@GOTPAGE
  ldr x0, [x0, _s2w@GOTPAGEOFF]
  ret

// The relocation addend points outside the named string. Named-symbol lookup must identify the
// string before applying the addend.
.globl _get_s2w_via_offset
_get_s2w_via_offset:
  adrp x0, _s1w@PAGE
  add x0, x0, _s1w@PAGEOFF+100
  sub x0, x0, #100
  ret

.subsections_via_symbols
