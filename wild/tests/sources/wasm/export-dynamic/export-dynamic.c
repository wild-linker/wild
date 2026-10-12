// --export-dynamic exports every defined, non-local, non-hidden symbol and keeps it live under
// --gc-sections. Data symbols are exported as globals holding their address.

//#AbstractConfig:base
//#CompArgs: -fvisibility=default -matomics
//#Object:export-dynamic2.c
//#ExpectSection: name
//#ExpectSym: _start
//#ExpectSym: explicit_export
//#NoSym: explicit_fn
//#NoSym: hidden_func
//#NoSym: weak_hidden_func
//#NoSym: local_func
//#NoSym: hidden_data

//#AbstractConfig:exported:base
//#ExpectSym: pub_func
//#ExpectSym: weak_func
//#ExpectSym: unreferenced_func
//#ExpectSym: other_func
//#ExpectSym: pub_data
//#ExpectSym: other_data
//#ExpectSym: tls_data
//#NoSym: unreferenced_hidden_func

//#Config:default:exported
//#LinkArgs: --export-dynamic
//#DoesNotContain: unreferenced_hidden_func

//#Config:short:exported
//#LinkArgs: -E

//#Config:no-gc-sections:exported
//#LinkArgs: --export-dynamic --no-gc-sections

// With `mutable-globals`, the linker-defined `__stack_pointer` is exported too. wasm-ld only does
// this since LLVM 22, so this is Wild-only.
//#Config:stack-pointer:base
//#LinkArgs: --export-dynamic
//#ReferenceLinkers:
//#ExpectSym: __stack_pointer

// Without `mutable-globals`, wasm-ld doesn't export the linker-defined `__stack_pointer`.
//#Config:mvp:base
//#CompArgs: -mcpu=mvp
//#LinkArgs: --export-dynamic
//#ExpectSym: pub_func
//#ExpectSym: pub_data
//#NoSym: __stack_pointer

//#Config:no-export-dynamic:base
//#LinkArgs: --export-dynamic --no-export-dynamic
//#NoSym: __stack_pointer
//#NoSym: pub_func
//#NoSym: weak_func
//#NoSym: pub_data
//#DoesNotContain: unreferenced_func

int pub_func(int x) { return x + 1; }

__attribute__((visibility("hidden"))) int hidden_func(int x) { return x * 2; }

static int local_func(int x) { return x - 1; }

// Also defined in export-dynamic2.c. Only one definition is kept and exported.
__attribute__((weak)) int weak_func(void) { return 42; }

__attribute__((weak, visibility("hidden"))) int weak_hidden_func(void) { return 43; }

int pub_data = 7;

__attribute__((visibility("hidden"))) int hidden_data = 8;

// Exported under its export name only.
__attribute__((export_name("explicit_export"))) int explicit_fn(void) { return 1; }

int unreferenced_func(void) { return 99; }

__attribute__((visibility("hidden"))) int unreferenced_hidden_func(void) { return 100; }

int other_func(void);

void _start(void) {
  int v = pub_data + hidden_data;
  v = hidden_func(v);
  v = local_func(v);
  v = pub_func(v);
  v += weak_func() + weak_hidden_func() + other_func();
  if (v != 123) {
    __builtin_trap();
  }
}
