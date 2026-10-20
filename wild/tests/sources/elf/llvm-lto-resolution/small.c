int data[10];
int initialized;
int entry1 __attribute__((used, section("entries"))) = 10;

__attribute__((constructor)) static void initialize(void) { initialized = 42; }

__attribute__((weak)) int override(void) { return 100; }

int call_override(void) { return override(); }
