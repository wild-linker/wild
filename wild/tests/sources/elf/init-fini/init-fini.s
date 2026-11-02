# Checks that -init and -fini set DT_INIT and DT_FINI to the named functions, that the tags are
# omitted if the named symbol isn't defined, and that without these options they point at .init
# and .fini.
//#AbstractConfig:default
//#Arch:x86_64
//#Mode:dynamic
//#RunEnabled:false

//#Config:no-flags:default
//#LinkArgs:-pie
//#ExpectDynamic:DT_INIT count=1
//#ExpectDynamic:DT_FINI count=1

//#Config:init-fini:default
//#LinkArgs:-pie -init my_init -fini my_fini
# The functions are in the middle of .text, at layout-dependent addresses, so linker-diff can't
# match the tag values between linkers.
//#DiffIgnore:.dynamic.DT_INIT
//#DiffIgnore:.dynamic.DT_FINI
//#ExpectDynamic:DT_INIT count=1
//#ExpectDynamic:DT_FINI count=1

# If the named symbol isn't defined, the tag is omitted, even though .init and .fini exist.
//#Config:missing:default
//#LinkArgs:-pie -init does_not_exist -fini does_not_exist
//#NoDynamic:DT_INIT
//#NoDynamic:DT_FINI

# Like glibc's crti.o: _init and _fini at the start of .init and .fini. bfd's default is
# `-init _init`, so it needs the symbols; Wild uses the sections, which start at the same address.
.section .init,"ax",@progbits
.globl _init
_init:
  ret

.section .fini,"ax",@progbits
.globl _fini
_fini:
  ret

.text
.globl _start
_start:
  ret

.globl my_init
my_init:
  ret

.globl my_fini
my_fini:
  ret
