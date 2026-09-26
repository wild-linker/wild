//#ReferenceLinkers:lld
//#ExpectError:(?i-u)Undefined symbol:? undefined_multi_a
//#ExpectError:.*multiple-undefined-symbols.c.o.*
//#ExpectError:(?i-u)Undefined symbol:? undefined_multi_b
//#ExpectError:(?i-u)Undefined symbol:? undefined_multi_c

int undefined_multi_a();
int undefined_multi_b();
int undefined_multi_c();

void _start(void) {
  undefined_multi_a();
  undefined_multi_b();
  undefined_multi_c();
}
