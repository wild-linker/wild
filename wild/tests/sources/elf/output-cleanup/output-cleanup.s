// Various configurations that check our output-file cleanup behaviour WRT various kinds of errors
// and flag combinations.

//#AbstractConfig:default
//#Arch:x86_64
//#Mode:static

// Linkers generally don't detect relocation overflows until after they've started writing the
// output file.
//#AbstractConfig:overflow:default
//#LinkArgs:--defsym=too_big=0x100000000
//#ExpectError:R_X86_64_32

//#Config:replace-mmap:overflow
//#WildExtraLinkArgs:--no-update-in-place --mmap-output-file --threads=1

//#Config:replace-buffered:overflow
//#WildExtraLinkArgs:--no-update-in-place --no-mmap-output-file --threads=1

//#Config:replace-parallel:overflow
//#WildExtraLinkArgs:--no-update-in-place --threads=2

//#Config:update-mmap:overflow
//#WildExtraLinkArgs:--update-in-place --mmap-output-file --threads=1

//#Config:update-buffered:overflow
//#WildExtraLinkArgs:--update-in-place --no-mmap-output-file --threads=1

// BFD removes the /dev/null symlink on failure; LLD preserves it.
//#Config:dev-null-replace:replace-buffered
//#ReferenceLinkers:lld
//#TestOutputToDevNull:true

//#AbstractConfig:invalid-options:default
//#LinkArgs:--defsym=too_big=42
// The incompatible options are specific to Wild.
//#ReferenceLinkers:
//#ExpectError:requires mmapped output

//#Config:invalid-options-replace:invalid-options
//#WildExtraLinkArgs:--no-update-in-place --no-mmap-output-file --madvise-huge-pages

//#Config:invalid-options-update:invalid-options
//#WildExtraLinkArgs:--update-in-place --no-mmap-output-file --madvise-huge-pages

//#AbstractConfig:dependency-file:default
//#LinkArgs:--defsym=too_big=42
//#ExpectError:Is a directory

//#Config:dependency-file-replace:dependency-file
//#LinkArgs:--dependency-file=$OUT_DIR
//#WildExtraLinkArgs:--no-update-in-place

.globl _start
.text
_start:
.long too_big
.section .note.GNU-stack,"",@progbits
