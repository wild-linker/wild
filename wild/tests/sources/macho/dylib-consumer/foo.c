#include <stdlib.h>
int value = 42;
int* volatile pointer = &value;
int foo(void) {
  void* allocation = malloc(16);
  if (!allocation) return 1;
  free(allocation);
  return *pointer;
}
