//#Object:runtime.c

__attribute__((weak)) extern int weak_undefined(void);

int main() {
  if (weak_undefined) {
    return weak_undefined();
  }
  return 42;
}
