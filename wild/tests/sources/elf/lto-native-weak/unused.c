// This archive member must not keep the unreferenced LTO definition alive.
__attribute__((weak)) int unused_provider(void) { return 7; }
