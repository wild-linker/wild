typedef int (*fn_t)(int);

__attribute__((noinline)) int callee(int x) { return x + 1; }

__attribute__((noinline)) int caller(int x) { return callee(x); }

__attribute__((noinline)) fn_t callee_address(void) { return callee; }
