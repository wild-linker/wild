//#LinkerDriver:clang
//#LinkArgs:-dynamiclib
//#Object:hidden.c
//#RunDynSym:foo
//#ExpectDynSym:_foo
//#NoDynSym:_hidden_value
//#ExpectSection:__thread_data
//#ExpectSection:__thread_bss
//#ExpectSection:__thread_vars

#include <pthread.h>

static _Thread_local volatile int initialized = 10;
static _Thread_local volatile int uninitialized;
extern _Thread_local volatile int hidden_value __attribute__((visibility("hidden")));

static int check_and_update(int expected_initialized, int expected_uninitialized,
                            int expected_hidden) {
  if (initialized != expected_initialized || uninitialized != expected_uninitialized ||
      hidden_value != expected_hidden)
    return 0;
  initialized += 1;
  uninitialized += 2;
  hidden_value += 3;
  return 1;
}

static void* worker(void* arg) {
  *(int*)arg = check_and_update(10, 0, 21) && check_and_update(11, 2, 24);
  return 0;
}

int foo(void) {
  if (!check_and_update(10, 0, 21) || !check_and_update(11, 2, 24)) return 1;

  pthread_t thread;
  int worker_ok = 0;
  if (pthread_create(&thread, 0, worker, &worker_ok) != 0) return 2;
  if (pthread_join(thread, 0) != 0) return 3;
  if (!worker_ok) return 4;

  if (!check_and_update(12, 4, 27)) return 5;
  return 42;
}
