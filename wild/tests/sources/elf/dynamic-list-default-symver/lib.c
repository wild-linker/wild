int f0_impl(void) { return 45; }
__asm__(".symver f0_impl, f0@@V1");

int f0(void);

int call_f0(void) { return f0(); }
