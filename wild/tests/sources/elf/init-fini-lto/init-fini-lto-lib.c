#include <unistd.h>

static int init_ran;

// Hidden, so that only DT_INIT and DT_FINI refer to these functions.
__attribute__((visibility("hidden"))) void lib_init(void) { init_ran = 1; }

// Runs at exit, after main has returned. Exits with 42 (success) only if lib_init also ran.
__attribute__((visibility("hidden"))) void lib_fini(void) { _exit(init_ran ? 42 : 2); }

int lib_init_ran(void) { return init_ran; }
