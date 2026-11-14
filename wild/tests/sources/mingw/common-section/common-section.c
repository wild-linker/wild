// A copy of elf/common-section test
//#CompArgs:-fcommon
//#Object:common_section0.c
//#Object:common_section1.c
//#Object:runtime.c
// TODO: LLD has a bug and puts `a` into `.data`
//#ReferenceLinkers:bfd
//#TestUpdateInPlace:true

int a;
extern int data[];
extern int q[];
extern int z[];

int main() {
  a = 30;
  q[0] = 20;
  z[0] = 40;
  // We have two declarations of `data`. One has size 10, the other 1000. The
  // linker should choose the one with the larger size.
  for (int i = 0; i < 1000; i++) {
    data[i] = 6;
  }
  // Try to detect if we've overflowed the space allocated to data. It's luck
  // whether the linker decides to put any of our canary variables after `data`,
  // but if we have enough of them, then there's a reasonable chance.
  if (a != 30 || q[0] != 20 || z[0] != 40) {
    return 101;
  }
  data[100] = 10;
  return 42;
}

//#ExpectSym: a section=".bss"
//#ExpectSym: data section=".bss"
//#ExpectSym: q section=".bss"
//#ExpectSym: z section=".bss"
