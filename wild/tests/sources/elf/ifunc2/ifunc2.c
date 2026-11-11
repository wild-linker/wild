//#AbstractConfig:default
//#DiffIgnore:section.data
//#DiffIgnore:section.data.alignment
//#DiffIgnore:section.rodata
//#DiffIgnore:section.rodata.alignment
//#RequiresGlibc:true

//#Config:pie:default
//#SkipArch:loongarch64
//#CompArgs:-fpie
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now
//#LinkArgs:-pie

//#Config:pie-loongarch64:pie
//#Arch:loongarch64
// LLD 21 produces crashing LoongArch IFUNC binaries, including static libc users.
// TODO: Remove the LoongArch BFD-only configurations once the oldest supported LLD is 22;
// these cases pass with LLD 22.
//#ReferenceLinkers:bfd

int exit_code = 2;

static void impl(void) { exit_code += 40; }
static void* resolver(void) { return impl; }
void* ifunc(void) __attribute__((ifunc("resolver")));

int main() {
  ifunc();
  return exit_code;
}
