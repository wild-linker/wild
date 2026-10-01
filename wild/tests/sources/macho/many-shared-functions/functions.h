#ifndef FUNCTION
#error "Define FUNCTION before including functions.h"
#endif

#define REPEAT_10(M) \
  M()                \
  M()                \
  M()                \
  M()                \
  M()                \
  M()                \
  M()                \
  M()                \
  M()                \
  M()

#define REPEAT_100(M) \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)        \
  REPEAT_10(M)

#define REPEAT_1000(M) \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)        \
  REPEAT_100(M)

#define REPEAT_10000(M) \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)        \
  REPEAT_1000(M)

REPEAT_10000(FUNCTION)
REPEAT_10000(FUNCTION)
REPEAT_10000(FUNCTION)
REPEAT_10000(FUNCTION)
REPEAT_10000(FUNCTION)
