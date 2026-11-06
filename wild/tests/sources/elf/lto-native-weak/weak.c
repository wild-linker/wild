__attribute__((weak)) int weak_fn(void) { return 7; }

int target(void) { return weak_fn(); }
