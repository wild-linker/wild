//#AbstractConfig:default
//#Arch:ppc64le
//#LinkerDriver:gcc
//#DiffEnabled:false

//#Config:pie:default
//#LinkArgs:-pie

//#Config:no-pie:default
//#LinkArgs:-no-pie

#include <stdio.h>

int main(void) {
  if (puts("one") < 0) {
    return 1;
  }
  if (puts("two") < 0) {
    return 2;
  }
  return 42;
}
