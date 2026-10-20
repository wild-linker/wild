// Note the __stack_pointer (in Global section) used to be present
// in older LLVM releases: (https://github.com/llvm/llvm-project/pull/199739).
//#ExpectSection: Memory
//#ExpectSym: memory
//#Contains: _start.command_export

static int x;

__attribute__((constructor(100))) static void init(void) { x = 42; }

void _start(void) {
  if (x != 42) {
    __builtin_trap();
  }
}
