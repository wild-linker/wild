__attribute__((weak)) int weak_func(void) { return 1; }

int other_data = 5;

_Thread_local int tls_data = 3;

int other_func(void) { return other_data + tls_data; }
