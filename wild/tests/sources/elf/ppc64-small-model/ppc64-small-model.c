//#Arch:ppc64le
//#LinkerDriver:gcc
//#DiffEnabled:false
//#Object:toc.s
//#CompArgs:-fPIE -mcmodel=small
//#LinkArgs:-pie

extern long load_toc16_ds(void);
extern long load_toc16(void);

int g = 11;

int main(void) {
  if (g != 11) {
    return 1;
  }
  if (load_toc16_ds() != 42) {
    return 2;
  }
  if (load_toc16() != 42) {
    return 3;
  }
  return 42;
}
