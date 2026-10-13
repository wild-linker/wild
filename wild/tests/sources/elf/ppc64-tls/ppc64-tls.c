//#AbstractConfig:default
//#Arch:ppc64le
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now -pie
//#CompArgs:-fPIE
//#DiffIgnore:section.rodata
//#DiffIgnore:section.data
//#DiffIgnore:section.data.alignment

//#Config:local-exec:default
//#CompArgs:-ftls-model=local-exec

//#Config:initial-exec:default
//#CompArgs:-ftls-model=initial-exec

//#Config:local-dynamic:default
//#CompArgs:-fPIC -ftls-model=local-dynamic

//#Config:global-dynamic:default
//#CompArgs:-fPIC -ftls-model=global-dynamic

//#Config:dso-initial-exec:default
//#CompArgs:-DPPC64_TLS_DSO
//#CompSoArgs:-fPIC -ftls-model=initial-exec
//#Shared:ppc64-tls-dso.c

//#Config:dso-global-dynamic:default
//#CompArgs:-DPPC64_TLS_DSO
//#CompSoArgs:-fPIC -ftls-model=global-dynamic
//#Shared:ppc64-tls-dso.c

#if defined(PPC64_TLS_DSO)
extern int ppc64_tls_dso_value(void);
#endif

static __thread long local_tls = 40;
static __thread char local_tls_byte = 2;

int main(void) {
#if defined(PPC64_TLS_DSO)
  return ppc64_tls_dso_value();
#else
  return (int)local_tls + local_tls_byte;
#endif
}
