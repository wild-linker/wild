// It'd be a lot better if we could use the C API rather than the C++ API, since that might allow us
// to dlopen libLTO. Unfortunately, the C API doesn't provide everything we need. e.g. we need to
// provide information not just about which symbols were preserved, but also how they were resolved
// - Prevailing, FinalDefinitionInLinkageUnit etc. We also need access to things like COMDAT info.
// The hook that we use for --wrap also doesn't seem to be available in the C API.

#include "llvm_lto.h"
#include "llvm/ADT/StringExtras.h"
#include "llvm/Config/llvm-config.h"
#include "llvm/IR/DiagnosticInfo.h"
#include "llvm/IR/DiagnosticPrinter.h"
#include "llvm/IR/Module.h"
#include "llvm/LTO/LTO.h"
#include "llvm/Pass.h"
#include "llvm/Support/CachePruning.h"
#include "llvm/Support/Caching.h"
#include "llvm/Support/SHA256.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/Threading.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/TargetParser/Triple.h"
#include <cstring>
#include <mutex>

using namespace llvm;

struct WildLlvmInput {
  // The bitcode bytes are borrowed from Wild until the LTO session is
  // destroyed.
  std::string name;
  std::unique_ptr<lto::InputFile> input;
};

struct WildLlvmOutput {
  SmallVector<char, 0> data;
  std::unique_ptr<MemoryBuffer> cached;
};

struct WildLlvmSession {
  std::vector<SmallVector<char, 0>> outputs;
  std::vector<std::unique_ptr<MemoryBuffer>> cached;
  std::string cache_dir;
  std::string cache_policy;
  std::mutex diagnostic_mutex;
  std::string diagnostics;
  std::string warnings;
  std::string cache_salt;
  std::unique_ptr<lto::LTO> lto;
};

static char *errorString(StringRef message) {
  char *result = new char[message.size() + 1];
  std::memcpy(result, message.data(), message.size());
  result[message.size()] = 0;
  return result;
}

static WildLlvmBytes bytes(StringRef value) {
  return {reinterpret_cast<const uint8_t *>(value.data()), value.size()};
}

const char *wild_llvm_version() { return LLVM_VERSION_STRING; }

void wild_llvm_error_destroy(char *error) { delete[] error; }

WildLlvmInput *wild_llvm_input_create(const uint8_t *data, size_t len,
                                      const char *name, char **error) {
  auto result = std::make_unique<WildLlvmInput>();
  result->name = name;

  auto input = lto::InputFile::create(MemoryBufferRef(
      StringRef(reinterpret_cast<const char *>(data), len), result->name));

  if (!input) {
    *error = errorString(toString(input.takeError()));
    return nullptr;
  }

  result->input = std::move(*input);
  Triple triple(result->input->getTargetTriple());

  if (!triple.isOSBinFormatELF()) {
    *error = errorString("LLVM LTO requires an ELF target");
    return nullptr;
  }

  return result.release();
}

void wild_llvm_input_destroy(WildLlvmInput *input) { delete input; }

size_t wild_llvm_symbol_count(const WildLlvmInput *input) {
  return input->input->symbols().size();
}

WildLlvmSymbol wild_llvm_symbol(const WildLlvmInput *input, size_t index) {
  const auto &symbol = input->input->symbols()[index];
  StringRef comdat;

  if (symbol.getComdatIndex() != -1)
    comdat = input->input->getComdatTable()[symbol.getComdatIndex()].first;

  uint32_t kind = symbol.isUndefined() ? (symbol.isWeak() ? 3 : 2)
                  : symbol.isCommon()  ? 4
                  : symbol.isWeak()    ? 1
                                       : 0;

  return {bytes(symbol.getName()),
          bytes(comdat),
          bytes(symbol.getSectionName()),
          symbol.isCommon() ? symbol.getCommonSize() : 0,
          kind,
          symbol.getVisibility() == GlobalValue::HiddenVisibility      ? 2u
          : symbol.getVisibility() == GlobalValue::ProtectedVisibility ? 3u
                                                                       : 0u,
          static_cast<uint32_t>(symbol.isUsed())};
}

WildLlvmBytes wild_llvm_triple(const WildLlvmInput *input) {
  return bytes(input->input->getTargetTriple());
}

void wild_llvm_libcalls([[maybe_unused]] const WildLlvmInput *input,
                        void (*callback)(void *, WildLlvmBytes),
                        void *context) {
#if LLVM_VERSION_MAJOR >= 19
  auto libcalls = lto::LTO::getRuntimeLibcallSymbols(
      Triple(input->input->getTargetTriple()));
#else
  auto libcalls = lto::LTO::getRuntimeLibcallSymbols();
#endif

  for (const char *name : libcalls)
    callback(context, bytes(name));
}

class WrapReferences : public ModulePass {
  std::vector<std::string> wrapped;

public:
  static char ID;

  explicit WrapReferences(std::vector<std::string> names)
      : ModulePass(ID), wrapped(std::move(names)) {}

  bool runOnModule(Module &m) override {
    // Full LTO can combine a reference and its definition in one object. Wild's
    // normal --wrap path redirects undefined symbols, so keep these references
    // explicitly directed at the wrapper even when the original is defined
    // here.
    for (const auto &name : wrapped) {
      GlobalValue *original = m.getNamedValue(name);

      if (!original || original->hasLocalLinkage())
        continue;

      std::string wrapperName = "__wrap_" + name;
      GlobalValue *wrapper = m.getNamedValue(wrapperName);

      if (!wrapper) {
        if (auto *type = dyn_cast<FunctionType>(original->getValueType())) {
          auto *function =
              Function::Create(type, GlobalValue::ExternalLinkage,
                               original->getAddressSpace(), wrapperName, &m);

          if (auto *old = dyn_cast<Function>(original))
            function->setCallingConv(old->getCallingConv());

          wrapper = function;
        } else {
          wrapper = new GlobalVariable(
              m, original->getValueType(), false, GlobalValue::ExternalLinkage,
              nullptr, wrapperName, nullptr, original->getThreadLocalMode(),
              original->getAddressSpace());
        }
      }

      // Aliases define another name for the original, not a reference to wrap.
      original->replaceUsesWithIf(wrapper, [](Use &use) {
        return !isa<GlobalAlias>(use.getUser());
      });
    }

    return true;
  }
};

char WrapReferences::ID = 0;

WildLlvmSession *wild_llvm_session_create(const WildLlvmConfig *config) {
  static std::once_flag initialized;

  std::call_once(initialized, [] {
    InitializeAllTargets();
    InitializeAllTargetMCs();
    InitializeAllAsmPrinters();
    InitializeAllAsmParsers();
  });

  auto session = std::make_unique<WildLlvmSession>();
  session->cache_dir = config->cache_dir;
  session->cache_policy = config->cache_policy;

  lto::Config conf;
  conf.OptLevel = config->opt_level;
  conf.CGOptLevel = CodeGenOptLevel::Default;
  conf.CPU = config->cpu;
  conf.Options.FunctionSections = true;
  conf.Options.DataSections = true;
  conf.Options.EmitAddrsig = true;

  if (config->relocation_model != 2)
    conf.RelocModel =
        config->relocation_model == 1 ? Reloc::PIC_ : Reloc::Static;

  conf.PTO.LoopVectorization = config->opt_level > 1;
  conf.PTO.SLPVectorization = config->opt_level > 1;

  conf.DiagHandler = [s = session.get()](const DiagnosticInfo &info) {
    if (info.getSeverity() == DS_Error || info.getSeverity() == DS_Warning) {
      std::lock_guard<std::mutex> guard(s->diagnostic_mutex);
      raw_string_ostream os(info.getSeverity() == DS_Error ? s->diagnostics
                                                           : s->warnings);
      DiagnosticPrinterRawOStream printer(os);
      info.print(printer);
      os << '\n';
    }
  };

  std::vector<std::string> wrapped;

  // LLVM hashes its own options, but cannot account for our custom codegen
  // pass.
  session->cache_salt = "wild-llvm-lto-v1";

  for (size_t i = 0; i < config->wrap_count; ++i) {
    wrapped.emplace_back(config->wrap[i]);
    session->cache_salt.push_back(0);
    session->cache_salt += config->wrap[i];
  }

  if (!wrapped.empty())
    conf.PreCodeGenPassesHook =
        [wrapped = std::move(wrapped)](legacy::PassManager &pm) {
          pm.add(new WrapReferences(wrapped));
        };

  session->lto = std::make_unique<lto::LTO>(
      std::move(conf), lto::createInProcessThinBackend(
                           heavyweight_hardware_concurrency(config->jobs)));

  return session.release();
}

void wild_llvm_session_destroy(WildLlvmSession *session) { delete session; }

char *wild_llvm_add(WildLlvmSession *session, WildLlvmInput *input,
                    const uint8_t *flags, size_t count) {
  std::vector<lto::SymbolResolution> resolutions(count);

  for (size_t i = 0; i < count; ++i) {
    resolutions[i].Prevailing = (flags[i] & 1) != 0;
    resolutions[i].FinalDefinitionInLinkageUnit = (flags[i] & 2) != 0;
    resolutions[i].VisibleToRegularObj = (flags[i] & 4) != 0;
    resolutions[i].ExportDynamic = (flags[i] & 8) != 0;
    resolutions[i].LinkerRedefined = (flags[i] & 16) != 0;
  }

  if (auto error = session->lto->add(std::move(input->input), resolutions))
    return errorString(toString(std::move(error)));

  return nullptr;
}

char *wild_llvm_run(WildLlvmSession *session) {
  auto policy = parseCachePruningPolicy(session->cache_policy);

  if (!policy)
    return errorString(toString(policy.takeError()));

  size_t count = session->lto->getMaxTasks();
  session->outputs.resize(count);
  session->cached.resize(count);

  AddStreamFn add =
      [session](unsigned task,
                const Twine &) -> Expected<std::unique_ptr<CachedFileStream>> {
    return std::make_unique<CachedFileStream>(
        std::make_unique<raw_svector_ostream>(session->outputs[task]));
  };

  FileCache cache;

  if (!session->cache_dir.empty()) {
    auto result = localCache("Wild ThinLTO", "wild", session->cache_dir,
                             [session](unsigned task, const Twine &,
                                       std::unique_ptr<MemoryBuffer> mb) {
                               session->cached[task] = std::move(mb);
                             });

    if (!result)
      return errorString(toString(result.takeError()));

    auto saltedCache =
        [local = std::move(*result), salt = session->cache_salt](
            unsigned task, StringRef key, const Twine &module) mutable {
          SHA256 hash;
          hash.update(salt);
          hash.update(key);
          return local(task, toHex(hash.final()), module);
        };

#if LLVM_VERSION_MAJOR >= 20
    cache = FileCache(std::move(saltedCache), session->cache_dir);
#else
    cache = std::move(saltedCache);
#endif
  }

  if (auto error = session->lto->run(add, cache))
    return errorString(toString(std::move(error)));

  if (!session->diagnostics.empty())
    return errorString(session->diagnostics);

  if (!session->cache_dir.empty()) {
#if LLVM_VERSION_MAJOR >= 23
    auto result = pruneCache(session->cache_dir, *policy, session->cached);
    if (!result)
      return errorString(toString(result.takeError()));
#else
    pruneCache(session->cache_dir, *policy, session->cached);
#endif
  }

  return nullptr;
}

size_t wild_llvm_output_count(const WildLlvmSession *session) {
  return session->outputs.size();
}

WildLlvmOutput *wild_llvm_take_output(WildLlvmSession *session, size_t index) {
  auto output = std::make_unique<WildLlvmOutput>();
  output->data = std::move(session->outputs[index]);
  output->cached = std::move(session->cached[index]);
  return output.release();
}

WildLlvmBytes wild_llvm_output_data(const WildLlvmOutput *output) {
  if (output->cached)
    return bytes(output->cached->getBuffer());

  return bytes(StringRef(output->data.data(), output->data.size()));
}

void wild_llvm_output_destroy(WildLlvmOutput *output) { delete output; }

WildLlvmBytes wild_llvm_warnings(const WildLlvmSession *session) {
  return bytes(session->warnings);
}
