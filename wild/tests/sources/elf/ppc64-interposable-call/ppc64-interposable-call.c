//#AbstractConfig:common
//#Arch:ppc64le
//#LinkerDriver:gcc
//#DiffEnabled:false
//#Shared:lib.c
//#CompSoArgs:-fPIC

//#Config:default:common

//#Config:symbolic:common
//#LinkSoArgs:-Wl,-Bsymbolic

int caller(int x);
int callee(int x);
typedef int (*fn_t)(int);
fn_t callee_address(void);

int main(void) {
  if (caller(1) != 2) {
    return 1;
  }
  if (callee(1) != 2) {
    return 2;
  }
  if (callee_address()(20) != 21) {
    return 3;
  }
  return 42;
}
