typedef int (*Func)(void);

static int implementation(void) { return 11; }
static Func resolve_foo(void) { return implementation; }
int foo(void) __attribute__((ifunc("resolve_foo")));

int call_foo(void) { return foo(); }
Func get_foo(void) { return foo; }
