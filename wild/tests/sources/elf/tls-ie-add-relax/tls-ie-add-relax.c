//#Arch:x86_64
//#CompArgs:-fpie
//#LinkerDriver:gcc
//#LinkArgs:-Wl,-z,now
//#LinkArgs:-pie
//#DiffIgnore:section.rodata

// Initial-exec TLS accesses of the form `add x@gottpoff(%rip), %reg`. In an executable, linkers
// relax these to local-exec: wild uses `add $x@tpoff, %reg`, GNU ld uses `lea x@tpoff(%reg),
// %reg`. linker-diff needs to accept both, for low and high (REX.B) registers.

__thread int tvar __attribute__((tls_model("initial-exec"))) = 40;

__asm__(
    ".text\n"
    ".globl get_tvar_rax\n"
    ".type get_tvar_rax, @function\n"
    "get_tvar_rax:\n"
    "    movq %fs:0, %rax\n"
    "    addq tvar@gottpoff(%rip), %rax\n"
    "    ret\n"
    ".size get_tvar_rax, .-get_tvar_rax\n"
    ".globl get_tvar_r9\n"
    ".type get_tvar_r9, @function\n"
    "get_tvar_r9:\n"
    "    movq %fs:0, %r9\n"
    "    addq tvar@gottpoff(%rip), %r9\n"
    "    movq %r9, %rax\n"
    "    ret\n"
    ".size get_tvar_r9, .-get_tvar_r9\n");

int *get_tvar_rax(void);
int *get_tvar_r9(void);

int main() {
  int *a = get_tvar_rax();
  int *b = get_tvar_r9();
  if (a != &tvar || b != &tvar) {
    return 1;
  }
  return *a + 2;
}
