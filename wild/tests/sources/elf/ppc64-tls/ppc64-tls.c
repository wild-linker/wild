//#AbstractConfig:default
//#Arch:ppc64le
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now
//#DiffIgnore:section.rodata
//#DiffIgnore:section.data
//#DiffIgnore:section.data.alignment

//#AbstractConfig:pie:default
//#CompArgs:-fPIE
//#LinkArgs:-pie

//#Config:local-exec:pie
//#CompArgs:-ftls-model=local-exec

//#Config:initial-exec:pie
//#CompArgs:-ftls-model=initial-exec

//#Config:local-dynamic:pie
//#CompArgs:-fPIC -ftls-model=local-dynamic

//#Config:global-dynamic:pie
//#CompArgs:-fPIC -ftls-model=global-dynamic

//#Config:local-exec-no-pie:default
//#CompArgs:-fno-pie -ftls-model=local-exec
//#LinkArgs:-no-pie

// A local-exec TLS reference inside the shared object itself. The thread-pointer offset is not
// known when linking a DSO. GNU ld accepts this with DF_STATIC_TLS and text relocations; wild
// rejects it.
//#Config:local-exec-shared:default
//#ReferenceLinkers:
//#CompArgs:-fPIC -ftls-model=local-exec
//#LinkArgs:-shared
//#ExpectError:cannot be used when making a shared object

//#Config:dso-initial-exec:pie
//#CompArgs:-DPPC64_TLS_DSO
//#CompSoArgs:-fPIC -ftls-model=initial-exec
//#Shared:ppc64-tls-dso.c

//#Config:dso-global-dynamic:pie
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
