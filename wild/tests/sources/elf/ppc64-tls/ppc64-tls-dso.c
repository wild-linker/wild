static __thread long dso_tls = 40;
static __thread char dso_tls_byte = 2;
__thread int imported_tls = 42;

int ppc64_tls_dso_value(void) { return (int)dso_tls + dso_tls_byte; }
