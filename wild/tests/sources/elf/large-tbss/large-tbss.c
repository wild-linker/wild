// LLD 23 removes the high-part TLSLE addition when the offset fits in 12 bits.
// TODO: Implement this relaxation in Wild.
//#DiffIgnore:rel.missing-opt.R_AARCH64_TLSLE_ADD_TPREL_HI12.ReplaceWithNop.*
//#LinkerDriver:gcc
//#DiffIgnore:section.rodata
//#DiffIgnore:section.data
//#CompArgs:-fPIE
//#LinkArgs:-pie

static __thread volatile unsigned char tls_zeros[0x22000];

int main(void) {
  if (tls_zeros[0] != 0 || tls_zeros[sizeof(tls_zeros) - 1] != 0) {
    return 2;
  }
  return 42;
}
