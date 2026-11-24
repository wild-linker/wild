//! Safe wrapper around our LLVM LTO C++ code.

use crate::FileSystem;
use crate::args::elf::ElfArgs;
use crate::bail;
use crate::elf::Elf;
use crate::elf::ElfClass;
use crate::ensure;
use crate::error::Context as _;
use crate::error::Result;
use crate::file_kind::FileKind;
use crate::grouping::Group;
use crate::grouping::SequencedInput;
use crate::input_data::FileLoader;
use crate::input_data::InputBytes;
use crate::input_data::InputRef;
use crate::input_data::LoadedInputs;
use crate::layout_rules::LayoutRulesBuilder;
use crate::lto::LtoInput;
use crate::lto::LtoInputInfo;
use crate::lto::LtoSymbol;
use crate::lto::SymbolKind;
use crate::output_kind::OutputKind;
use crate::output_section_id::OutputSections;
use crate::parsing::InternalSymDefInfo;
use crate::parsing::ParsedInputObject;
use crate::parsing::SymbolPlacement;
use crate::platform::Args as _;
use crate::resolution::ResolvedFile;
use crate::resolution::ResolvedGroup;
use crate::resolution::Resolver;
use crate::symbol::UnversionedSymbolName;
use crate::symbol_db::SymbolDb;
use crate::value_flags::FlagsForSymbol;
use crate::value_flags::PerSymbolFlags;
use crate::value_flags::ValueFlags;
use bumpalo_herd::Herd;
use hashbrown::HashMap;
use hashbrown::HashSet;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_char;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[repr(C)]
struct Bytes {
    data: *const u8,
    len: usize,
}

impl Bytes {
    // Only use inside the owner wrappers, which bind the resulting slice's lifetime
    // to the input, session or output that owns these bytes.
    unsafe fn as_slice<'a>(&self) -> &'a [u8] {
        if self.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.len) }
        }
    }
}

#[repr(C)]
struct RawSymbol {
    name: Bytes,
    comdat: Bytes,
    section: Bytes,
    size: u64,
    kind: u32,
    visibility: u32,
    used: u32,
}

#[repr(C)]
struct Config {
    opt_level: u32,
    jobs: u32,
    relocation_model: u32,
    cpu: *const c_char,
    cache_dir: *const c_char,
    cache_policy: *const c_char,
    wrap: *const *const c_char,
    wrap_count: usize,
}

unsafe extern "C" {
    fn wild_llvm_version() -> *const c_char;

    fn wild_llvm_input_create(
        data: *const u8,
        len: usize,
        name: *const c_char,
        error: *mut *mut c_char,
    ) -> *mut c_void;

    fn wild_llvm_input_destroy(input: *mut c_void);

    fn wild_llvm_symbol_count(input: *const c_void) -> usize;

    fn wild_llvm_symbol(input: *const c_void, index: usize) -> RawSymbol;

    fn wild_llvm_triple(input: *const c_void) -> Bytes;

    fn wild_llvm_libcalls(
        input: *const c_void,
        callback: unsafe extern "C" fn(*mut c_void, Bytes),
        context: *mut c_void,
    );

    fn wild_llvm_session_create(config: *const Config) -> *mut c_void;

    fn wild_llvm_session_destroy(session: *mut c_void);

    fn wild_llvm_add(
        session: *mut c_void,
        input: *mut c_void,
        resolutions: *const u8,
        count: usize,
    ) -> *mut c_char;

    fn wild_llvm_run(session: *mut c_void) -> *mut c_char;

    fn wild_llvm_warnings(session: *const c_void) -> Bytes;

    fn wild_llvm_output_count(session: *const c_void) -> usize;

    fn wild_llvm_take_output(session: *mut c_void, index: usize) -> *mut c_void;

    fn wild_llvm_output_data(output: *const c_void) -> Bytes;

    fn wild_llvm_output_destroy(output: *mut c_void);

    fn wild_llvm_error_destroy(error: *mut c_char);
}

struct Input<'data> {
    raw: NonNull<c_void>,
    _data: PhantomData<&'data [u8]>,
}

struct InputSymbol<'input> {
    name: &'input [u8],
    comdat: &'input [u8],
    size: u64,
    kind: SymbolKind,
    visibility: u8,
    used: bool,
    has_section: bool,
}

impl<'data> Input<'data> {
    fn new(data: &'data [u8], name: &CStr) -> Result<Self> {
        let mut error = std::ptr::null_mut();
        let raw = unsafe {
            wild_llvm_input_create(data.as_ptr(), data.len(), name.as_ptr(), &raw mut error)
        };

        check_error(error)?;

        Ok(Self {
            raw: NonNull::new(raw).context("LLVM returned no input")?,
            _data: PhantomData,
        })
    }

    fn triple(&self) -> &[u8] {
        unsafe { wild_llvm_triple(self.raw.as_ptr()).as_slice() }
    }

    fn symbol_count(&self) -> usize {
        unsafe { wild_llvm_symbol_count(self.raw.as_ptr()) }
    }

    fn symbols(&self) -> impl ExactSizeIterator<Item = InputSymbol<'_>> {
        (0..self.symbol_count()).map(|index| {
            let symbol = unsafe { wild_llvm_symbol(self.raw.as_ptr(), index) };

            InputSymbol {
                name: unsafe { symbol.name.as_slice() },
                comdat: unsafe { symbol.comdat.as_slice() },
                size: symbol.size,
                kind: match symbol.kind {
                    0 => SymbolKind::Def,
                    1 => SymbolKind::WeakDef,
                    2 => SymbolKind::Undef,
                    3 => SymbolKind::WeakUndef,
                    4 => SymbolKind::Common,
                    _ => unreachable!(),
                },
                visibility: symbol.visibility as u8,
                used: symbol.used != 0,
                has_section: symbol.section.len != 0,
            }
        })
    }

    fn runtime_libcalls(&self) -> Vec<Vec<u8>> {
        unsafe extern "C" fn collect(context: *mut c_void, name: Bytes) {
            let names = unsafe { &mut *context.cast::<Vec<Vec<u8>>>() };
            names.push(unsafe { name.as_slice() }.to_vec());
        }

        let mut names = Vec::<Vec<u8>>::new();

        unsafe {
            wild_llvm_libcalls(self.raw.as_ptr(), collect, (&raw mut names).cast());
        }

        names
    }
}

impl Drop for Input<'_> {
    fn drop(&mut self) {
        unsafe {
            wild_llvm_input_destroy(self.raw.as_ptr());
        }
    }
}

struct Session<'data> {
    raw: NonNull<c_void>,
    inputs: Vec<Input<'data>>,
}

impl<'data> Session<'data> {
    fn new(options: &Options, output_kind: OutputKind, wrap: &[CString]) -> Result<Self> {
        let wrap_pointers: Vec<_> = wrap.iter().map(|s| s.as_ptr()).collect();
        let config = Config {
            opt_level: options.opt_level,
            jobs: options.jobs,
            relocation_model: if output_kind.is_partial_link() {
                2
            } else if output_kind.is_position_independent() {
                1
            } else {
                0
            },
            cpu: options.cpu.as_ptr(),
            cache_dir: options.cache_dir.as_ptr(),
            cache_policy: options.cache_policy.as_ptr(),
            wrap: wrap_pointers.as_ptr(),
            wrap_count: wrap_pointers.len(),
        };

        // The bridge copies the configuration strings during creation.
        let raw = unsafe { wild_llvm_session_create(&raw const config) };

        Ok(Self {
            raw: NonNull::new(raw).context("LLVM returned no session")?,
            inputs: Vec::new(),
        })
    }

    fn add(&mut self, input: Input<'data>, resolutions: &[u8]) -> Result {
        assert_eq!(resolutions.len(), input.symbol_count());

        // LLVM consumes the module, but still borrows the input's name and bitcode.
        self.inputs.push(input);
        let input = self.inputs.last().unwrap();

        check_error(unsafe {
            wild_llvm_add(
                self.raw.as_ptr(),
                input.raw.as_ptr(),
                resolutions.as_ptr(),
                resolutions.len(),
            )
        })
    }

    fn run(self, on_warning: impl FnOnce(&[u8])) -> Result<Vec<Output>> {
        let result = check_error(unsafe { wild_llvm_run(self.raw.as_ptr()) });
        let warnings = unsafe { wild_llvm_warnings(self.raw.as_ptr()).as_slice() };

        if !warnings.is_empty() {
            on_warning(warnings);
        }

        result?;

        (0..unsafe { wild_llvm_output_count(self.raw.as_ptr()) })
            .map(|index| {
                NonNull::new(unsafe { wild_llvm_take_output(self.raw.as_ptr(), index) })
                    .map(Output)
                    .context("LLVM returned no output")
            })
            .collect()
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        // Destroy LLVM before dropping the inputs whose storage it borrows.
        unsafe {
            wild_llvm_session_destroy(self.raw.as_ptr());
        }
    }
}

#[derive(Debug)]
pub(crate) struct Output(NonNull<c_void>);

// Outputs own immutable buffers or cache mappings, independent of the LLVM
// session. They can be read on any thread and destroyed after those reads end.
unsafe impl Send for Output {}

unsafe impl Sync for Output {}

impl Output {
    pub(crate) fn bytes(&self) -> &[u8] {
        unsafe { wild_llvm_output_data(self.0.as_ptr()).as_slice() }
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        unsafe {
            wild_llvm_output_destroy(self.0.as_ptr());
        }
    }
}

fn check_error(error: *mut c_char) -> Result {
    if !error.is_null() {
        let message = unsafe { CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned();

        unsafe {
            wild_llvm_error_destroy(error);
        }

        bail!("LLVM {}: {message}", version());
    }

    Ok(())
}

pub(crate) fn version() -> &'static str {
    unsafe { CStr::from_ptr(wild_llvm_version()) }
        .to_str()
        .unwrap_or("unknown")
}

pub(crate) struct LlvmLto<'data> {
    args: &'data ElfArgs,
    herd: &'data Herd,
    inputs: Vec<Option<Input<'data>>>,
    libcalls_by_target: HashMap<Vec<u8>, Vec<Vec<u8>>>,
    options: Options,
}

#[derive(Default)]
struct Options {
    opt_level: u32,
    jobs: u32,
    cpu: CString,
    cache_dir: CString,
    cache_policy: CString,
}

impl<'data> LlvmLto<'data> {
    pub(crate) fn new(args: &'data ElfArgs, herd: &'data Herd) -> Result<Self> {
        let mut options = Options {
            opt_level: 2,
            jobs: u32::try_from(
                args.common
                    .num_threads
                    .map_or_else(rayon::current_num_threads, std::num::NonZeroUsize::get),
            )?,
            ..Options::default()
        };

        for option in &args.plugin_args {
            let option = option.to_str()?;

            if let Some(level) = option.strip_prefix('O') {
                options.opt_level = level.parse().context("Invalid LTO optimization level")?;

                ensure!(
                    options.opt_level <= 3,
                    "LTO optimization level must be between 0 and 3"
                );
            } else if let Some(jobs) = option.strip_prefix("jobs=") {
                options.jobs = if jobs == "all" {
                    u32::try_from(std::thread::available_parallelism()?.get())?
                } else {
                    jobs.parse().context("Invalid ThinLTO jobs value")?
                };
            } else if let Some(value) = option.strip_prefix("cache-dir=") {
                options.cache_dir = CString::new(value)?;
            } else if let Some(value) = option.strip_prefix("cache-policy=") {
                options.cache_policy = CString::new(value)?;
            } else if let Some(value) = option.strip_prefix("mcpu=") {
                options.cpu = CString::new(value)?;
            } else if option != "thinlto" {
                bail!("Unsupported LLVM LTO option: --plugin-opt={option}");
            }
        }

        Ok(Self {
            args,
            herd,
            inputs: Vec::new(),
            libcalls_by_target: HashMap::new(),
            options,
        })
    }

    pub(crate) fn is_initialised(&self) -> bool {
        !self.inputs.is_empty()
    }

    pub(crate) fn process_input(
        &mut self,
        input_ref: InputRef<'data>,
        kind: FileKind,
    ) -> Result<Box<LtoInputInfo<'data>>> {
        ensure!(
            kind == FileKind::LlvmIr,
            "Input file {input_ref} contains {kind}, which requires a linker plugin"
        );

        let name = CString::new(format!("{}#{}", input_ref, self.inputs.len()))?;
        let input = Input::new(input_ref.data(), &name)
            .with_context(|| format!("Failed to read {input_ref}"))?;
        let triple = input.triple();

        self.libcalls_by_target
            .entry_ref(triple)
            .or_insert_with(|| input.runtime_libcalls());

        let arena = self.herd.get();

        let symbols = input
            .symbols()
            .map(|symbol| {
                let name = arena.alloc_slice_copy(symbol.name);
                let comdat = symbol.comdat;

                LtoSymbol {
                    name: UnversionedSymbolName::new(name),
                    version: None,
                    comdat_key: (!comdat.is_empty()).then(|| {
                        let name: &[u8] = arena.alloc_slice_copy(comdat);
                        crate::hash::PreHashed::new(name, crate::hash::hash_bytes(name))
                    }),
                    visibility: symbol.visibility,
                    kind: Some(symbol.kind),
                    size: symbol.size,
                }
            })
            .collect();

        let backend_index = self.inputs.len();
        self.inputs.push(Some(input));

        Ok(Box::new(LtoInputInfo {
            input_ref,
            symbols,
            handle: None,
            backend_index,
        }))
    }

    pub(crate) fn all_symbols_read<F: FileSystem, C: ElfClass>(
        &mut self,
        symbol_db: &mut SymbolDb<'data, Elf<C>>,
        resolver: &mut Resolver<'data, Elf<C>>,
        file_loader: &FileLoader<'data, F>,
        flags: &mut PerSymbolFlags,
        output_sections: &mut OutputSections<'data, Elf<C>>,
        layout_rules_builder: &mut LayoutRulesBuilder<'data>,
    ) -> Result {
        if !crate::lto::has_loaded_lto_input(&resolver.resolved_groups) {
            return Ok(());
        }

        crate::timing_phase!("LLVM LTO codegen");

        let libcalls = self
            .libcalls_by_target
            .values()
            .flatten()
            .map(Vec::as_slice);

        crate::lto::resolve_roots(symbol_db, resolver, flags, libcalls)?;

        crate::lto::prepare_symbols_for_lto(symbol_db, flags, &resolver.resolved_groups);

        let loaded =
            self.generate_inputs(symbol_db, &resolver.resolved_groups, flags, file_loader)?;

        crate::lto::ingest_generated_inputs(
            loaded,
            symbol_db,
            resolver,
            flags,
            output_sections,
            layout_rules_builder,
        )
    }

    fn generate_inputs<F: FileSystem, C: ElfClass>(
        &mut self,
        symbol_db: &SymbolDb<'data, Elf<C>>,
        resolved_groups: &[ResolvedGroup<'data, Elf<C>>],
        flags: &PerSymbolFlags,
        file_loader: &FileLoader<'data, F>,
    ) -> Result<LoadedInputs<'data, Elf<C>>> {
        // Redirects are applied during layout, so a canonical IR definition can still be
        // overridden by --defsym or a linker-script assignment.
        let mut redirected = HashSet::new();

        let mut record_redirects = |definitions: &[InternalSymDefInfo<'data, Elf<C>>]| {
            for definition in definitions {
                if let SymbolPlacement::Redirect(redirect) = &definition.placement
                    && !redirect.is_provide()
                {
                    redirected.insert(definition.name);
                }
            }
        };

        for group in &symbol_db.groups {
            match group {
                Group::Prelude(prelude) => record_redirects(&prelude.symbol_definitions),
                Group::LinkerScripts(scripts) => {
                    for script in scripts {
                        record_redirects(&script.parsed.symbol_defs);
                    }
                }
                _ => {}
            }
        }

        let wrap = self
            .args
            .wrap
            .iter()
            .map(|s| CString::new(s.as_str()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut session = Session::new(&self.options, symbol_db.output_kind, &wrap)?;

        for group in resolved_groups {
            for file in &group.files {
                let ResolvedFile::LtoInput(file) = file else {
                    continue;
                };

                let Group::LtoInputs(inputs) = &symbol_db.groups[file.file_id.group()] else {
                    unreachable!();
                };

                let object = &inputs[file.file_id.file()];
                let input = self.inputs[object.backend_index].take().unwrap();

                let resolutions =
                    self.symbol_resolutions(object, &input, symbol_db, flags, &redirected);

                session
                    .add(input, &resolutions)
                    .with_context(|| format!("Failed to add {} to LLVM LTO", object.input_ref))?;
            }
        }

        let outputs = session.run(|warnings| {
            self.args.warning(format!(
                "LLVM {}: {}",
                version(),
                String::from_utf8_lossy(warnings)
            ));
        })?;

        let mut loaded = LoadedInputs {
            objects: Vec::new(),
            linker_scripts: Vec::new(),
            stub_libraries: Vec::new(),
            lto_objects: Vec::new(),
        };

        for (index, output) in outputs.into_iter().enumerate() {
            if output.bytes().is_empty() {
                continue;
            }

            let input = file_loader.generated_input(output, index, self.args)?;
            let data = input.data();
            let modifiers = input.file.modifiers;

            loaded.objects.push(ParsedInputObject::new(
                &InputBytes {
                    input,
                    data,
                    modifiers,
                    kind: FileKind::ElfObject,
                },
                self.args,
            ));
        }

        Ok(loaded)
    }

    fn symbol_resolutions<C: ElfClass>(
        &self,
        object: &LtoInput<'data>,
        input: &Input<'_>,
        symbol_db: &SymbolDb<'data, Elf<C>>,
        flags: &PerSymbolFlags,
        redirected: &HashSet<&[u8]>,
    ) -> Vec<u8> {
        object
            .symbols_iter()
            .zip(input.symbols())
            .map(|((id, symbol), metadata)| {
                let wrapped = self
                    .args
                    .wrap
                    .iter()
                    .any(|name| name.as_bytes() == symbol.name.bytes());

                let definition = if wrapped {
                    let real = format!("__real_{}", String::from_utf8_lossy(symbol.name.bytes()));

                    symbol_db
                        .get_unversioned(&UnversionedSymbolName::prehashed(real.as_bytes()))
                        .map(|id| symbol_db.definition(id))
                } else {
                    Some(symbol_db.definition(id))
                };

                let is_redirected = redirected.contains(symbol.name.bytes());

                let redefined = wrapped
                    || is_redirected
                    || definition.is_some_and(|definition| {
                        !definition.is_undefined()
                            && matches!(
                                symbol_db.file(symbol_db.file_id_for_symbol(definition)),
                                SequencedInput::Prelude(_)
                                    | SequencedInput::LinkerScript(_)
                                    | SequencedInput::SyntheticSymbols(_)
                            )
                    });

                let prevailing = !is_redirected && symbol.is_definition() && definition == Some(id);
                let symbol_flags = flags.flags_for_symbol(id);
                let exported = symbol_flags.contains(ValueFlags::EXPORT_DYNAMIC);

                let visible = self.args.should_output_partial_object()
                    || redefined
                    || metadata.used
                    || metadata.has_section
                    || exported
                    || symbol_flags.contains(ValueFlags::HAS_NON_IR_REF);

                let final_definition = !is_redirected
                    && definition.is_some_and(|definition| {
                        !definition.is_undefined()
                            && matches!(
                                symbol_db.file(symbol_db.file_id_for_symbol(definition)),
                                SequencedInput::LtoInput(_)
                            )
                            && (self.args.should_output_executable()
                                || symbol.visibility != object::elf::STV_DEFAULT.0)
                    });

                u8::from(prevailing)
                    | (u8::from(final_definition) << 1)
                    | (u8::from(visible) << 2)
                    | (u8::from(exported) << 3)
                    | (u8::from(redefined) << 4)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Input;
    use super::version;

    #[test]
    fn malformed_bitcode_reports_an_error_and_can_be_retried() {
        for _ in 0..2 {
            let bytes = b"BC\xc0\xde\0\0\0\0";
            let error = Input::new(bytes, c"broken.o")
                .err()
                .expect("invalid bitcode");
            assert!(error.to_string().contains(&format!("LLVM {}", version())));
        }
    }
}
