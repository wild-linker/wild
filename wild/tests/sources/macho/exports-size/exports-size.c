//#AbstractConfig:default
//#LinkerDriver:clang
//#ExpectDynSym:_exported_prefix_000
//#ExpectDynSym:_exported_prefix_999

//#Config:executable:default

//#Config:dylib:default
//#LinkArgs:-dynamiclib
//#RunDynSym:exported_prefix_999

// Enough exports to cross ULEB128 offset and code-signature page boundaries when the
// preliminary reservation is replaced with the final trie size.
#define EXPORT(n) \
  int exported_prefix_##n(void) { return 42; }
#define TEN(n) \
  EXPORT(n##0) \
  EXPORT(n##1) \
  EXPORT(n##2) \
  EXPORT(n##3) \
  EXPORT(n##4) \
  EXPORT(n##5) \
  EXPORT(n##6) \
  EXPORT(n##7) \
  EXPORT(n##8) \
  EXPORT(n##9)
#define HUNDRED(n) \
  TEN(n##0)        \
  TEN(n##1)        \
  TEN(n##2)        \
  TEN(n##3)        \
  TEN(n##4)        \
  TEN(n##5)        \
  TEN(n##6)        \
  TEN(n##7)        \
  TEN(n##8)        \
  TEN(n##9)

HUNDRED(0)
HUNDRED(1)
HUNDRED(2)
HUNDRED(3)
HUNDRED(4)
HUNDRED(5)
HUNDRED(6)
HUNDRED(7)
HUNDRED(8)
HUNDRED(9)

int main(void) { return exported_prefix_000() + exported_prefix_999() - 42; }
