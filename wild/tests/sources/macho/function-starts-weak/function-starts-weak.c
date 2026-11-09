//#AbstractConfig:default
//#LinkerDriver:clang
// Wild still emits superseded weak bodies. Their local labels may add function starts;
// allow those entries until dead stripping removes the bodies too.
//#ExpectFunctionStarts:_main _function ...

//#Config:weak-duplicates:default
//#Object:weak.c
//#Object:weak-copy.c
// Wild currently calls the weak definition directly rather than through an interposable stub.
//#DiffIgnore:section.__stubs
//#DiffIgnore:section.__got

//#Config:private-weak-duplicates:default
//#Object:weak.c:-fvisibility=hidden
//#Object:weak-copy.c:-fvisibility=hidden

//#Config:weak-then-strong:default
//#Object:weak.c
//#Object:strong.c

//#Config:strong-then-weak:default
//#Object:strong.c
//#Object:weak.c

int function(void);

int main(void) { return function(); }
