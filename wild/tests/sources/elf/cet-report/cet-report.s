# Tests for CET (Control-flow Enforcement Technology) reporting flags.
# These flags are x86-64 specific and control how Wild reports missing
# IBT (Indirect Branch Tracking) and SHSTK (Shadow Stack) properties
# in input object files.
#
# When -z force-ibt is used, Wild additionally adds the IBT property
# to the output binary, matching lld behavior.
//#AbstractConfig:default
//#Arch:x86_64
//#Mode:static
//#RunEnabled:false
//#CompArgs:-fcf-protection=none

//#Config:force-ibt:default
//#ReferenceLinkers:lld
//#LinkArgs:-z force-ibt
//#ExpectWarning:.*: -z force-ibt: file does not have GNU_PROPERTY_X86_FEATURE_1_IBT property
//#ExpectSectionBytes:.note.gnu.property=0x020000c00400000001000000 16..28

//#Config:force-ibt-mixed:force-ibt
//#Object:marked.s

//#Config:cet-report-warning:default
//#ReferenceLinkers:lld
//#LinkArgs:-z cet-report=warning
//#ExpectWarning:.*: -z cet-report: file does not have GNU_PROPERTY_X86_FEATURE_1_IBT property
//#ExpectWarning:.*: -z cet-report: file does not have GNU_PROPERTY_X86_FEATURE_1_SHSTK property

//#Config:cet-report-error:default
//#ReferenceLinkers:lld
//#LinkArgs:-z cet-report=error
//#ExpectError:.*: -z cet-report: file does not have GNU_PROPERTY_X86_FEATURE_1_IBT property

//#Config:cet-report-none:default
//#ReferenceLinkers:lld
//#LinkArgs:-z cet-report=none

//#Config:cet-report-invalid:default
//#ReferenceLinkers:lld
//#LinkArgs:-z cet-report=invalid
//#ExpectErrorWild:unknown -z cet-report= value 'invalid'

.globl _start
_start:
  ret
