// Checks that the -1 and 0 marker words in crtbegin's and crtend's .ctors/.dtors stay in their own
// sections, as with GNU ld's default linker script, rather than being merged into
// .init_array/.fini_array, where they'd be called as functions. GCC built without .init_array
// support provides such crtbegin/crtend files; these stand-ins have the same sections.
// lld keeps all of .ctors separate, so on glibc it doesn't run legacy_ctor; only GNU ld is used as
// a reference.
//#LinkerDriver:gcc
//#CompArgs:-fPIE
//#LinkArgs:-pie
//#ReferenceLinkers:bfd
//#Object:as(crtbegin.o):crtbegin.s
//#Object:as(crtend.o):crtend.s
// The markers must survive garbage collection, in order: crtbegin's -1, then crtend's 0.
//#ExpectSectionBytes:.ctors=0xffffffffffffffff0000000000000000
//#ExpectSectionBytes:.dtors=0xffffffffffffffff0000000000000000

static int ran;

static void legacy_ctor(void) { ran = 1; }

__attribute__((section(".ctors"), used)) static void (*const legacy_ctor_ptr)(void) = legacy_ctor;

int main(void) { return ran ? 42 : 1; }
