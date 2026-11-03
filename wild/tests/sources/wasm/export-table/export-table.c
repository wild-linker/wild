//#Config:default
//#ExpectSection:Table
//#ExpectSection:Element
//#ExpectSym:_start
//#ExpectSym:memory
//#NoSym:__indirect_function_table

//#Config:export-table
//#LinkArgs:--export-table
//#ExpectSection:Table
//#ExpectSection:Element
//#ExpectSym:_start
//#ExpectSym:memory
//#ExpectSym:__indirect_function_table

static int inc(int x) { return x + 1; }

typedef int (*Fn)(int);

void _start(void) {
  Fn f = inc;
  if (f(1) != 2) {
    __builtin_trap();
  }
}
