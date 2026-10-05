//#LinkerDriver:clang
//#LinkArgs:-Wl,-headerpad_max_install_names
//#ExpectLoadCommandSize:LC_LOAD_DYLIB 56
//#ExpectLoadCommandSize:LC_UUID 24
//#ExpectLoadCommandSize:LC_MAIN 24
//#ExpectLoadCommandSize:LC_SEGMENT_64 72,312,152,72

#include <stdlib.h>

int main(void) { exit(42); }
