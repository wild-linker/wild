int dependency(void);

int excluded_data = 3;

int excluded(void) { return dependency(); }

#ifndef NO_WEAK_OVERRIDE
__attribute__((weak)) int included(void) { return 100; }
#endif
