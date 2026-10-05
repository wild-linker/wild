// Verify that AArch64 build attributes are merged into
// .note.gnu.property during a relocatable link.
//
// All three inputs support BTI.
// PAC and GCS are not supported by every input, so FEATURE_1_AND
// must contain only BTI.
//
// The PAuth ABI platform and schema are identical in all inputs and
// must be emitted as GNU_PROPERTY_AARCH64_FEATURE_PAUTH.
//
//#Arch:aarch64
//#Compiler:clang
//#ReferenceLinkers:
//#LinkArgs:-r
//#RunEnabled:false
//#DiffEnabled:false
//#Object:input2.s
//#Object:input3.s
//#ExpectSection:.note.gnu.property
//#NoSection:.ARM.attributes
//#ExpectSectionBytes:.note.gnu.property=0x040000002800000005000000474e5500000000c0040000000100000000000000010000c01000000031000000000000001300000000000000

.text
.globl main_input
main_input:
    ret

.aeabi_subsection aeabi_pauthabi, required, uleb128
.aeabi_attribute Tag_PAuth_Platform, 49
.aeabi_attribute Tag_PAuth_Schema, 19

.aeabi_subsection aeabi_feature_and_bits, optional, uleb128
.aeabi_attribute Tag_Feature_BTI, 1
.aeabi_attribute Tag_Feature_PAC, 1
.aeabi_attribute Tag_Feature_GCS, 1
