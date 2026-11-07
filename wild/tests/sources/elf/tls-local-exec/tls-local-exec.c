//#AbstractConfig:default
// LLD 23 removes the high-part TLSLE addition when the offset fits in 12 bits.
// TODO: Implement this relaxation in Wild.
//#DiffIgnore:rel.missing-opt.R_AARCH64_TLSLE_ADD_TPREL_HI12.ReplaceWithNop.*
//#DiffIgnore:section.data
//#DiffIgnore:section.data.alignment
//#DiffIgnore:section.rodata
//#DiffIgnore:section.rodata.alignment

//#Config:pie:default
//#CompArgs:-fpie
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now
//#LinkArgs:-pie

__thread long tvar = 1;
__thread int tvar2 = 2;
__thread char tvar3 = 3;

int main() {
  // __builtin_printf ("%ld, %d, %d\n", tvar, tvar2, tvar3);
  return tvar + tvar2 + tvar3 + 36;
}
