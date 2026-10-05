//#LinkerDriver:clang
// Other linkers may reserve padding after the load commands instead of inside them.
//#ReferenceLinkers:
//#LinkArgs:-Wl,-headerpad_max_install_names
// sizeof(dylib_command) (24) + MAXPATHLEN (1024), already aligned to 8 bytes.
//#ExpectLoadCommandSize:LC_LOAD_DYLIB 1048
//#ExpectLoadCommandSize:LC_UUID 24
//#ExpectLoadCommandSize:LC_MAIN 24
//#ExpectLoadCommandSize:LC_SEGMENT_64 72,312,152,72

#include <stdlib.h>

int main(void) { exit(42); }
