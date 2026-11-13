//#Object:runtime.c
//#Object:crt0.c
//#Object:crt1.c
//#TestUpdateInPlace:true

void crt_ctor(void) {}

extern void (*const crt_xcb0)(void);
extern void (*const crt_xcb1)(void);

__attribute__((used, section(".CRT$XC"))) void (*const crt_xc)(void) = crt_ctor;
__attribute__((used, section(".CRT$XD"))) void (*const crt_xd)(void) = crt_ctor;
__attribute__((used, section(".CRT$X"))) void (*const crt_x)(void) = crt_ctor;
__attribute__((used, section(".CRT$XCA"))) void (*const crt_xca)(void) = crt_ctor;

int main() {
  if (&crt_x > &crt_xc) return 100;
  if (&crt_xc > &crt_xca) return 101;
  if (&crt_xca > &crt_xd) return 102;

  if (&crt_xcb0 > &crt_xcb1) return 110;
  if (&crt_xca > &crt_xcb0) return 111;
  if (&crt_xcb1 > &crt_xd) return 112;
  return 42;
}
