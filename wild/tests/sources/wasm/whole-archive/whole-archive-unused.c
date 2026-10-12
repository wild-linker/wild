extern int side_effect;

__attribute__((used)) void unused_kept_fn(void) {}

void unused_dead_fn(void) {}

__attribute__((constructor)) static void set_side_effect(void) { side_effect = 1; }
