//#AbstractConfig:default
//#Arch:ppc64le
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now

//#AbstractConfig:pie:default
//#LinkArgs:-pie

// Verify Wild drops `__tls_get_addr` from `.dynsym` after GD/LD -> LE relaxation (LLD retains an
// unused undefined entry).
//#Config:gd-pie:pie
//#CompArgs:-fPIC -ftls-model=global-dynamic
//#ReferenceLinkers:bfd
//#NoDynSym:__tls_get_addr

//#Config:ld-pie:pie
//#CompArgs:-fPIC -ftls-model=local-dynamic
//#ReferenceLinkers:bfd
//#NoDynSym:__tls_get_addr

//#Config:gd-static:default
//#CompArgs:-fPIC -ftls-model=global-dynamic
//#LinkArgs:-static

//#Config:ld-static:default
//#CompArgs:-fPIC -ftls-model=local-dynamic
//#LinkArgs:-static

//#Config:gd-static-no-relax:default
//#CompArgs:-fPIC -ftls-model=global-dynamic
//#LinkArgs:-static -Wl,--no-relax

//#Config:gd-noplt:pie
//#CompArgs:-fPIC -fno-plt -ftls-model=global-dynamic
//#ReferenceLinkers:bfd

//#Config:gd-small:pie
//#CompArgs:-fPIC -mcmodel=small -ftls-model=global-dynamic
//#ReferenceLinkers:bfd
//#NoDynSym:__tls_get_addr

//#Config:gd-no-relax:pie
//#CompArgs:-fPIC -ftls-model=global-dynamic
//#LinkArgs:-Wl,--no-relax

// Exceeds 32 KiB in `.tdata` so `R_PPC64_TPREL16_HA` is non-zero for variables placed after it.
static __thread unsigned char large_tls_pad[40000] = {1};
static __thread long local_tls = 40;
static __thread char local_tls_byte = 2;

int main(void) { return (int)local_tls + local_tls_byte + large_tls_pad[0] - 1; }
