//#Config:default
//#Arch:ppc64le
//#Compiler:clang
//#LinkerDriver:gcc
//#CompArgs:-fPIC -O2 -ftls-model=initial-exec
//#LinkArgs:-pie -Wl,-z,now

__thread char ie_c = 1;
__thread short ie_s = 2;
__thread int ie_i = 10;
__thread long ie_l = 20;
__thread float ie_f = 4.0f;
__thread double ie_d = 5.0;

__attribute__((noinline)) void write_types(int delta) {
  ie_c += delta;
  ie_s += delta;
  ie_i += delta;
  ie_l += delta;
  ie_f += (float)delta;
  ie_d += (double)delta;
}

__attribute__((noinline)) int read_types(void) {
  return ie_c + ie_s + ie_i + (int)ie_l + (int)ie_f + (int)ie_d;
}

int main(void) {
  write_types(0);
  return read_types();
}
