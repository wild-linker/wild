//#AbstractConfig:default
//#LinkerDriver:gcc
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

__thread unsigned long tls_var_a = 0x1122334455667788UL;
__thread char tls_var_b[8] __attribute__((aligned(256)));

int main(void) {
  if (tls_var_a != 0x1122334455667788UL) {
    return 1;
  }
  if (tls_var_b[0] != 0) {
    return 2;
  }
  if (((unsigned long)tls_var_b) & 255UL) {
    return 3;
  }
  return 42;
}
