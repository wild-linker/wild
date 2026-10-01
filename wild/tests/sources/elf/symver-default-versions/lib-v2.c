int foo_v2(void) { return 22; }
__asm__(".symver foo_v2,foo@@V2");
