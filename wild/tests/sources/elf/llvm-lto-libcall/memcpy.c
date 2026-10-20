void* memcpy(void* d, const void* s, unsigned long n) {
  volatile unsigned char* dst = d;
  const volatile unsigned char* src = s;

  for (unsigned long i = 0; i < n; ++i) dst[i] = src[i];

  return d;
}
