int foo(void) { return 10; }

int alias(void) __attribute__((alias("foo")));
