//#AbstractConfig:default
//#Arch:ppc64le
//#LinkerDriver:gcc
//#LinkArgs:-pie -Wl,-z,now
//#CompSoArgs:-fPIC
//#Shared:dso.c
//#ReferenceLinkers:bfd

//#Config:medium:default
//#CompArgs:-fPIC -ftls-model=global-dynamic
//#NoDynSym:__tls_get_addr

//#Config:small:default
//#CompArgs:-fPIC -mcmodel=small -ftls-model=global-dynamic
//#NoDynSym:__tls_get_addr

//#Config:noplt:default
//#CompArgs:-fPIC -fno-plt -ftls-model=global-dynamic

extern __thread int imported_tls;

int main(void) { return imported_tls; }
