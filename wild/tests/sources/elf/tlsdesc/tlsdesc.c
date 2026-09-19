// TODO: remove DiffIgnore for asm.* once relaxations are supported

//#AbstractConfig:default
//#DiffIgnore:.dynamic.DT_NEEDED
//#DiffIgnore:section.data
//#DiffIgnore:section.rodata

//#AbstractConfig:gcc-tls-desc-base:default
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now
//#Object:tlsdesc-obj.c
//#Arch: x86_64

//#Config:gcc-tls-desc:gcc-tls-desc-base
//#CompArgs:-mtls-dialect=gnu2 -fPIC -O2

//#Config:gcc-tls-desc-desc:gcc-tls-desc-base
//#CompArgs:-mtls-dialect=desc -fPIC
//#SkipArch: x86_64,riscv64,ppc64le

//#Config:gcc-tls-desc-pie:gcc-tls-desc-base
//#CompArgs:-mtls-dialect=gnu2 -fPIE

//#Config:gcc-tls-desc-pie-desc:gcc-tls-desc-base
//#CompArgs:-mtls-dialect=desc
//#SkipArch: x86_64,riscv64,ppc64le

//#Config:gcc-tls-desc-static:gcc-tls-desc-base
//#CompArgs:-mtls-dialect=gnu2 -fPIC -static
//#CompSoArgs:-mtls-dialect=gnu2 -fPIC -static
//#Shared:tlsdesc-obj.c
//#DiffIgnore:asm.get_value
//#Arch: x86_64

//#AbstractConfig:gcc-tls-desc-shared-base:gcc-tls-desc-base
//#Shared:tlsdesc-obj.c
//#Arch: x86_64

//#Config:gcc-tls-desc-shared:gcc-tls-desc-shared-base
//#CompArgs:-mtls-dialect=gnu2 -fPIC
//#CompSoArgs:-mtls-dialect=gnu2 -fPIC

//#Config:gcc-tls-desc-shared-desc:gcc-tls-desc-shared-base
//#CompArgs:-mtls-dialect=desc
//#CompSoArgs:-mtls-dialect=desc
//#SkipArch: x86_64,riscv64,loongarch64,ppc64le

//#AbstractConfig:clang-tls-desc-base:gcc-tls-desc-base
//#Compiler:clang
//#RequiresCompilerFlags:-mtls-dialect=gnu2
//#Arch: x86_64

//#Config:clang-tls-desc:clang-tls-desc-base
//#CompArgs:-mtls-dialect=gnu2 -fPIC

//#Config:clang-tls-desc-desc:clang-tls-desc-base
//#CompArgs:-mtls-dialect=desc -fPIC
//#SkipArch: x86_64,riscv64

//#AbstractConfig:clang-tls-desc-shared-base:clang-tls-desc-base
//#Shared:tlsdesc-obj.c
//#Arch: x86_64

//#Config:clang-tls-desc-shared:clang-tls-desc-shared-base
//#CompArgs:-mtls-dialect=gnu2 -fPIC
//#CompSoArgs:-mtls-dialect=gnu2 -fPIC

//#Config:clang-tls-desc-shared-desc:clang-tls-desc-shared-base
//#CompArgs:-mtls-dialect=desc -fPIC
//#CompSoArgs:-mtls-dialect=desc -fPIC
//#SkipArch: x86_64,riscv64

int get_value();

int main() { return get_value(); }
