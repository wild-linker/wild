int state;

static void first(void) { state = state * 10 + 1; }
static void second(void) { state = state * 10 + 2; }

__attribute__((
    used,
    section("__DATA,__mod_init_func,mod_init_funcs"))) void (*const first_registration)(void) =
    first;
__attribute__((
    used,
    section("__DATA,__mod_init_func,mod_init_funcs"))) void (*const second_registration)(void) =
    second;
