#include <unistd.h>

// Keep libSystem loaded even when the fixture uses -dead_strip_dylibs.
int wild_test_forwarded(void) { return getpid() > 0 ? 42 : 0; }
int wild_test_not_forwarded(void) { return 7; }
