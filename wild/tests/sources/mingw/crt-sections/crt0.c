void crt_ctor();

__attribute__((used, section(".CRT$XCB"))) void (*const crt_xcb0)(void) = crt_ctor;
