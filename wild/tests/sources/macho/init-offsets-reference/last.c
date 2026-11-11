extern int state;
int last_state;

// References to registration storage must not change constructor order.
__attribute__((constructor)) static void last_initializer(void) { last_state = state; }
