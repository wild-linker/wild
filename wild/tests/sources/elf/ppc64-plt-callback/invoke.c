static volatile int value = 1;

int invoke(int (*callback)(void)) {
  int before = value;
  int result = callback();
  return result + value - before;
}
