#if PROTECTED == 1
#define VISIBILITY __attribute__((visibility("protected")))
#else
#define VISIBILITY
#endif

VISIBILITY __thread int first = 11;
VISIBILITY __thread int second = 42;

int entry(void) {
  if (first != 11) {
    return 1;
  }
  return second;
}
