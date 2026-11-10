extern int other_fn(void);

int pulled_fn(void) { return other_fn(); }

__attribute__((used)) void member_kept(void) {}

void member_dead(void) {}
