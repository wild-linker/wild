.section __TEXT,__cstring,cstring_literals
.p2align 0

.globl _s1h
_s1h:
  .asciz "Hello"

.globl _s1w
_s1w:
  .asciz "World"

// Regular, 8-byte-aligned data must not be treated as a cstring section.
.section __TEXT,__regular_data
.p2align 3

.globl _a1
_a1:
  .asciz "Aligned"

// Strings in the same custom cstring section can merge with one another, but not with identical
// strings in other sections.
.section __TEXT,__custom_cstring,cstring_literals
.p2align 0

.globl _s3h
_s3h:
  .asciz "Hello"

.globl _noref
_noref:
  .asciz "No reference to this string"

// An empty cstring section must be accepted without producing output bytes.
.section __TEXT,__empty_cstring,cstring_literals
.globl _in_empty_cstring
_in_empty_cstring:

.subsections_via_symbols
