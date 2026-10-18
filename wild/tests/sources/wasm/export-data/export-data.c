//#AbstractConfig:layout
//#LinkArgs:--stack-first

//#Config:default:layout
//#LinkArgs:--export=exported_data --export=exported_fn --export=hidden_data
//#ExpectSym:exported_data address=65536
//#ExpectSym:hidden_data address=65544
//#ExpectSym:exported_fn
//#ExpectSym:_start
//#NoSym:plain_data
//#NoSym:unreferenced_data
//#NoSym:dead_data
//#DoesNotContain:dead_data

//#Config:if-defined:layout
//#LinkArgs:--export-if-defined=exported_data --export-if-defined=does_not_exist
//#ExpectSym:exported_data address=65536
//#NoSym:does_not_exist
//#NoSym:plain_data
//#NoSym:hidden_data

// An unreferenced data symbol stays live when `--export` names it.
//#Config:gc:layout
//#LinkArgs:--export=unreferenced_data
//#ExpectSym:unreferenced_data address=65548
//#ExpectSym:_start
//#NoSym:dead_data
//#DoesNotContain:dead_data

int exported_data = 42;

int plain_data = 1;

__attribute__((visibility("hidden"))) int hidden_data = 3;

int unreferenced_data = 7;

int dead_data = 9;

int exported_fn(void) { return exported_data + hidden_data; }

void _start(void) {
  if (exported_fn() != 45 || plain_data != 1) {
    __builtin_trap();
  }
}
