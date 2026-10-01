int main(void) {
  long long sum = 0;

#define CALL_FUNCTION_EXPANDED(n) \
  {                               \
    extern int f##n(void);        \
    sum += f##n();                \
  }
#define CALL_FUNCTION(n) CALL_FUNCTION_EXPANDED(n)
#define FUNCTION() CALL_FUNCTION(__COUNTER__)
#include "functions.h"

  return sum == (49999LL * 50000LL) / 2 ? 42 : 1;
}
