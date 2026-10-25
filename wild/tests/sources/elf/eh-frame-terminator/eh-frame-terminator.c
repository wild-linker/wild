//#Object:terminator.s
//#Object:frame-after-terminator.s
//#Object:runtime.c
//#LinkArgs:--eh-frame-hdr
// BFD has different behaviour here. Unlike wild and lld, it preserves frame data after a
// terminator.
//#ReferenceLinkers:lld
//#DoesNotContain:SHOULD_NOT_APPEAR_IN_BINARY
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

void _start(void) {
  runtime_init();
  exit_syscall(42);
}
