// Verify merging AArch64 build attributes with GNU property notes during a
// relocatable link.
//
// One input uses build attributes and two inputs use GNU properties.
// The first two inputs support BTI, PAC and GCS.
// The second GNU property input supports only BTI and PAC, so the merged
// FEATURE_1_AND value must contain only BTI and PAC.
// PAuth information from the build attributes and GNU property must agree.
//
//#Arch:aarch64
//#Compiler:clang
//#ReferenceLinkers:
//#LinkArgs:-r
//#RunEnabled:false
//#DiffEnabled:false
//#Object:merged-property.s
//#Object:merged-property2.s
//#ExpectSection:.note.gnu.property
//#NoSection:.ARM.attributes
//#ExpectSectionBytes:.note.gnu.property=0x040000002800000005000000474e5500000000c0040000000300000000000000010000c01000000031000000000000001300000000000000

.aeabi_subsection aeabi_pauthabi, required, uleb128
.aeabi_attribute Tag_PAuth_Platform, 49
.aeabi_attribute Tag_PAuth_Schema, 19

.aeabi_subsection aeabi_feature_and_bits, optional, uleb128
.aeabi_attribute Tag_Feature_BTI, 1
.aeabi_attribute Tag_Feature_PAC, 1
.aeabi_attribute Tag_Feature_GCS, 1
