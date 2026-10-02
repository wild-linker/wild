//#AbstractConfig:default
//#LinkerDriver:gcc
//#Object:ptr_black_box.c
//#SkipArch:ppc64le
//#DiffIgnore:section.tdata.alignment
//#DiffIgnore:section.rodata
//#DiffIgnore:section.rela.plt.link
//#DiffIgnore:section.rela.dyn
//#DiffIgnore:section.data
//#DiffIgnore:section.sdata
//#DiffIgnore:rel.extra-got-plt-got

//#Config:dynamic:default

//#Config:static:default
//#LinkArgs:-static

//#Config:compress-debug:default
//#CompArgs:-g
//#LinkArgs:-Wl,--compress-debug-sections=zlib
//#DiffIgnore:section.debug_*

//#Config:compress-debug-static:static
//#CompArgs:-g
//#LinkArgs:-Wl,--compress-debug-sections=zlib
//#DiffIgnore:section.debug_*

#include "../common/ptr_black_box.h"

__thread unsigned long tls_var_a = 0x1122334455667788UL;
__thread char tls_var_b[8] __attribute__((aligned(256)));

int main(void) {
  if (tls_var_a != 0x1122334455667788UL) {
    return 1;
  }
  if (tls_var_b[0] != 0) {
    return 2;
  }
  if (ptr_to_int(tls_var_b) % 256 != 0) {
    return 3;
  }
  return 42;
}
