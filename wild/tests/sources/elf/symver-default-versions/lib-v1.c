int foo_v1(void) { return 11; }
__asm__(".symver foo_v1,foo@@V1");
