//#Object:weak-data2.c

__attribute__((weak)) int value = 0;
int* ptr = &value;

void _start(void) {
  if (value != 42) {
    __builtin_trap();
  }
  if (*ptr != 42) {
    __builtin_trap();
  }
}
