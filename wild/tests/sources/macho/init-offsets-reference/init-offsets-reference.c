//#LinkerDriver:clang
//#LinkArgs:-Wl,-dead_strip
//#Object:initializer.c
//#Object:last.c
//#NoSection:__mod_init_func
//#ExpectSection:__init_offsets

extern int state;
extern int last_state;
extern void (*const first_registration)(void);
extern void (*const second_registration)(void);

// Like rustc's symbols.o, reference constructor registrations in another object
// without dereferencing them. The second registration has a nonzero section offset.
// The link must succeed and the initializers must run, whether the linker resolves
// these references or discards the unused anchors during dead stripping.
void (*const* anchors[])(void) = {&first_registration, &second_registration};

static int offset_state;
__attribute__((constructor)) static void offset_initializer(void) { offset_state = 7; }

int main(void) {
  if (state != 12 || offset_state != 7 || last_state != 12) return 1;
  return 42;
}
