#ifndef WILD_LLVM_LTO_H
#define WILD_LLVM_LTO_H

#include <stddef.h>
#include <stdint.h>

typedef struct WildLlvmInput WildLlvmInput;
typedef struct WildLlvmSession WildLlvmSession;
typedef struct WildLlvmOutput WildLlvmOutput;

typedef struct WildLlvmBytes {
  const uint8_t *data;
  size_t len;
} WildLlvmBytes;

typedef struct WildLlvmSymbol {
  WildLlvmBytes name;
  WildLlvmBytes comdat;
  WildLlvmBytes section;
  uint64_t size;
  uint32_t kind;
  uint32_t visibility;
  // llvm.used
  uint32_t used;
} WildLlvmSymbol;

typedef struct WildLlvmConfig {
  uint32_t opt_level;
  uint32_t jobs;
  uint32_t relocation_model;
  const char *cpu;
  const char *cache_dir;
  const char *cache_policy;
  const char *const *wrap;
  size_t wrap_count;
} WildLlvmConfig;

// Inputs borrow bitcode bytes until the session is destroyed. Symbol views are
// valid until add consumes the input. Taking an output transfers ownership to
// the caller; its byte view lives until wild_llvm_output_destroy. Non-null error
// strings must be released with wild_llvm_error_destroy.
#ifdef __cplusplus
extern "C" {
#endif

const char *wild_llvm_version();

WildLlvmInput *wild_llvm_input_create(const uint8_t *, size_t, const char *,
                                      char **);

void wild_llvm_input_destroy(WildLlvmInput *);

size_t wild_llvm_symbol_count(const WildLlvmInput *);

WildLlvmSymbol wild_llvm_symbol(const WildLlvmInput *, size_t);

WildLlvmBytes wild_llvm_triple(const WildLlvmInput *);

// Invokes the callback synchronously; each name is borrowed for that call.
void wild_llvm_libcalls(const WildLlvmInput *,
                        void (*callback)(void *, WildLlvmBytes), void *context);

WildLlvmSession *wild_llvm_session_create(const WildLlvmConfig *);

void wild_llvm_session_destroy(WildLlvmSession *);

// Resolution bits: prevailing, final, visible to native code, exported,
// redefined.
char *wild_llvm_add(WildLlvmSession *, WildLlvmInput *, const uint8_t *,
                    size_t);

char *wild_llvm_run(WildLlvmSession *);

WildLlvmBytes wild_llvm_warnings(const WildLlvmSession *);

size_t wild_llvm_output_count(const WildLlvmSession *);

WildLlvmOutput *wild_llvm_take_output(WildLlvmSession *, size_t);

WildLlvmBytes wild_llvm_output_data(const WildLlvmOutput *);

void wild_llvm_output_destroy(WildLlvmOutput *);

void wild_llvm_error_destroy(char *);

#ifdef __cplusplus
}
#endif
#endif
