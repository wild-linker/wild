int cpu_model = 17;
__asm__(".symver cpu_model,cpu_model@V1");

int versioned_v1(void) { return 1; }
int versioned_v2(void) { return 2; }
__asm__(".symver versioned_v1,versioned@V1");
__asm__(".symver versioned_v2,versioned@@V2");
