#define DEFINE_FUNCTION_EXPANDED(n) \
  int f##n(void) { return n; }
#define DEFINE_FUNCTION(n) DEFINE_FUNCTION_EXPANDED(n)
#define FUNCTION() DEFINE_FUNCTION(__COUNTER__)
#include "functions.h"
