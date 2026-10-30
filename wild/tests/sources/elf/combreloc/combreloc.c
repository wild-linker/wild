//#AbstractConfig:base
//#Arch:x86_64
//#RequiresGlibc:true
//#CompArgs:-fPIC
//#Object:relative.s
//#LinkArgs:-shared -z now
//#RunDynSym:check_relocations
//#ReferenceLinkers:bfd,lld
//#DiffIgnore:.dynamic.DT_RELA*
//#DiffIgnore:rel.R_X86_64_64.R_X86_64_64

//#Config:enabled:base
//#CompArgs:-DSORTED=1
//#LinkArgs:-z combreloc

//#Config:enable-last:enabled
//#LinkArgs:-z nocombreloc -z combreloc

//#Config:disabled:base
//#CompArgs:-DSORTED=0
//#LinkArgs:-z nocombreloc

//#Config:disable-last:disabled
//#LinkArgs:-z combreloc -z nocombreloc

//#Config:default:base
//#CompArgs:-DSORTED=0
//#ReferenceLinkers:

#include <elf.h>
#include <stddef.h>
#include <stdint.h>

extern const Elf64_Dyn _DYNAMIC[] __attribute__((visibility("hidden")));
extern const char __ehdr_start[] __attribute__((visibility("hidden")));
extern int* relative_pointers[2];

int foo = 10;
int bar = 20;
static int local = 30;
int* references[] = {&foo, &bar, &foo, &local, &local, &foo};

static int implementation(void) { return 42; }
static int (*resolver(void))(void) { return implementation; }
static int indirect(void) __attribute__((ifunc("resolver")));
int (*indirect_pointer)(void) = indirect;

int check_relocations(void) {
  const Elf64_Rela* relocations = NULL;
  size_t count = 0;
  size_t relative_count = 0;
  for (const Elf64_Dyn* d = _DYNAMIC; d->d_tag != DT_NULL; ++d) {
    if (d->d_tag == DT_RELA) {
      // glibc adjusts DT_RELA to its runtime address.
      relocations = (const Elf64_Rela*)d->d_un.d_ptr;
    } else if (d->d_tag == DT_RELASZ) {
      count = d->d_un.d_val / sizeof(Elf64_Rela);
    } else if (d->d_tag == DT_RELACOUNT) {
      relative_count = d->d_un.d_val;
    }
  }
  if (relocations == NULL || count == 0 || relative_count > count) {
    return 1;
  }

  const uintptr_t start = (uintptr_t)references - (uintptr_t)__ehdr_start;
  size_t reference_count = 0;
  int saw_irelative = 0;
  for (size_t i = 0; i < count; ++i) {
    const Elf64_Rela* r = &relocations[i];
    unsigned type = ELF64_R_TYPE(r->r_info);
    unsigned symbol = ELF64_R_SYM(r->r_info);
    if (SORTED) {
      if (i < relative_count) {
        if (type != R_X86_64_RELATIVE || (i > 0 && r->r_offset < relocations[i - 1].r_offset)) {
          return 2;
        }
      } else if (type == R_X86_64_IRELATIVE) {
        saw_irelative = 1;
      } else {
        if (saw_irelative) {
          return 3;
        }
        // BFD and LLD order symbol groups differently, but both keep them together.
        for (size_t j = relative_count; j < i; ++j) {
          if (ELF64_R_SYM(relocations[j].r_info) == symbol &&
              (ELF64_R_SYM(relocations[i - 1].r_info) != symbol ||
               relocations[j].r_offset > r->r_offset)) {
            return 4;
          }
        }
      }
    }
    if (type == R_X86_64_64 && r->r_offset >= start && r->r_offset < start + sizeof(references)) {
      ++reference_count;
    }
  }
  if (reference_count != 4 || (SORTED && relative_count < 2)) {
    return 6;
  }
  if (*references[0] != 10 || *references[1] != 20 || *references[2] != 10 ||
      *references[3] != 30 || *references[4] != 30 || *references[5] != 10) {
    return 7;
  }
  if (*relative_pointers[0] != 31 || *relative_pointers[1] != 31) {
    return 8;
  }
  return indirect_pointer();
}
