//#Config:default
//#LinkArgs:--export=__table_base
//#ExpectSym:__table_base address=1
//#ExpectTable:max=initial

//#Config:offset
//#LinkArgs:--table-base=100 --export=__table_base
//#ExpectSym:__table_base address=100
//#ExpectTable:max=initial

// `--table-base=0` is unset and becomes 1.
//#Config:zero
//#LinkArgs:--table-base=0 --export=__table_base
//#ExpectSym:__table_base address=1
//#ExpectTable:max=initial

static int inc(int x) { return x + 1; }

typedef int (*Fn)(int);

void _start(void) {
  Fn f = inc;
  if (f(1) != 2) {
    __builtin_trap();
  }
}
