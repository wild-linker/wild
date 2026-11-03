//#AbstractConfig:default
// LLD 23 removes the high-part TLSLE addition when the offset fits in 12 bits.
// TODO: Implement this relaxation in Wild.
//#DiffIgnore:rel.missing-opt.R_AARCH64_TLSLE_ADD_TPREL_HI12.ReplaceWithNop.*
//#LinkerDriver:gcc
//#DiffIgnore:section.rodata
//#DiffIgnore:section.data
//#DiffIgnore:section.data.alignment
//#ExpectSym:main

//#Config:gcc:default
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:gcc-static:default
// LLD uses lea for defined symbols and leaves undefined-weak GOT loads unrelaxed.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.static-non-pie
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-non-pie
// Alpine's compiler driver produces PIE even with -static. TODO: pass -no-pie automatically
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-pie
//#SkipArch:loongarch64
//#LinkArgs:-static -Wl,--gc-sections
//#DiffIgnore:section.sdata
// TODO: RISC-V BFD linker keeps the symbol in .dynsym section
//#DiffIgnore:section.rela.dyn
//#DiffIgnore:rel.extra-got-plt-got
//#CompArgs:-fPIE

//#Config:gcc-static-pie-no-relax:default
//#CompArgs:-fPIE
//#LinkArgs:-static-pie -Wl,--gc-sections -Wl,--no-relax
//#DiffEnabled:false
//#ReferenceLinkers:
// TODO: #874
//#SkipArch: riscv64,ppc64le

//#Config:clang-static:default
// LLD uses lea for defined symbols and leaves undefined-weak GOT loads unrelaxed.
//#DiffIgnore:rel.different-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).MovIndirectToLea.static-non-pie
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-non-pie
// Alpine's compiler driver produces PIE even with -static. TODO: pass -no-pie automatically
//#DiffIgnore:rel.extra-opt.R_X86_64_REX_GOTPCRELX.RexMovIndirectToAbsolute(3).static-pie
//#SkipArch:loongarch64
//#Compiler:clang
//#LinkArgs:-static
//#DiffIgnore:section.sdata
//#DiffIgnore:rel.extra-got-plt-got
// LoongArch GNU ld puts static IFUNC relocations in .rela.dyn instead of .rela.plt.
//#DiffIgnore:section.rela.dyn
//#CompArgs:-fPIE

//#Config:clang-static-pie-no-relax:default
//#Compiler:clang
//#CompArgs:-fPIE
//#LinkArgs:-static-pie -Wl,--gc-sections -Wl,--no-relax
//#DiffEnabled:false
//#ReferenceLinkers:
// For some reason, both linkers cannot find: `rcrt1.o`
//#SkipArch: riscv64,ppc64le

//#Config:clang:default
//#Compiler: clang
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:gcc-indirect-external:default
//#CompArgs:-fPIE -mno-direct-extern-access
//#RequiresCompilerFlags:-mno-direct-extern-access
//#LinkArgs:-pie

//#Config:gcc-static-loongarch64:gcc-static
//#Arch:loongarch64
// LLD 21 produces crashing LoongArch IFUNC binaries, including static libc users.
// TODO: Remove the LoongArch BFD-only configurations once the oldest supported LLD is 22;
// these cases pass with LLD 22.
//#ReferenceLinkers:bfd

//#Config:clang-static-loongarch64:clang-static
//#Arch:loongarch64
// LLD 21 produces crashing LoongArch IFUNC binaries, including static libc users.
//#ReferenceLinkers:bfd

int main() { return 42; }
