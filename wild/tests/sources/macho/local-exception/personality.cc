#include <stdint.h>
#include <stdlib.h>
#include <unwind.h>

extern "C" _Unwind_Reason_Code __gxx_personality_v0(int, _Unwind_Action, uint64_t,
                                                    _Unwind_Exception*, _Unwind_Context*) {
  _Exit(42);
}
