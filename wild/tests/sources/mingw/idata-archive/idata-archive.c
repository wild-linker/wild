//#Object:runtime.c
//#ReferenceLinkers:bfd,lld
//#Archive:as(libfoo.a):s0.s,h.s,t.s

__declspec(dllimport) int IsDebuggerPresent(void);

int main() {
  IsDebuggerPresent();
  return 42;
}
