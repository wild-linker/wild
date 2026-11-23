//#AbstractConfig:default
//#Arch:ppc64le
//#LinkerDriver:gcc
//#Object:foo.c
// lld does not decode R_PPC64_PLT16_* / PLTSEQ / PLTCALL.
//#ReferenceLinkers:bfd

//#Config:pie:default
//#CompArgs:-fPIE -fno-plt
//#LinkArgs:-pie

//#Config:no-pie:default
//#CompArgs:-fPIE -fno-plt
//#LinkArgs:-no-pie

int foo(void);

int main(void) {
  if (foo() != 40) {
    return 1;
  }
  return 42;
}
