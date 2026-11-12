use crate::FileSystem;
use crate::alignment;
use crate::alignment::Alignment;
use crate::arch::Architecture;
use crate::args::coff::CoffArgs;
use crate::bail;
use crate::coff_writer;
use crate::error::Result;
use crate::file_kind::FileKind;
use crate::input_data::InputBytes;
use crate::input_data::InputRef;
use crate::layout;
use crate::layout::HandlerData as _;
use crate::layout::Resolution;
use crate::layout::SectionGcUnit;
use crate::layout::SymbolCopyInfo;
use crate::layout_rules;
use crate::layout_rules::SectionKind;
use crate::layout_rules::SectionRule;
use crate::output_section_id::OutputOrderBuilder;
use crate::output_section_id::OutputSectionId;
use crate::output_section_id::SectionIdentity;
use crate::output_section_id::SectionName;
use crate::output_section_id::SectionOutputInfo;
use crate::output_section_part_map::OutputSectionPartMap;
use crate::parsing::InternalSymDefInfo;
use crate::parsing::SymbolPlacement;
use crate::part_id::PartId;
use crate::platform;
use crate::platform::Arch;
use crate::platform::Args;
use crate::platform::CommonSymbol;
use crate::platform::ObjectFile as _;
use crate::platform::Platform;
use crate::platform::RawSymbolName as _;
use crate::platform::Relocation;
use crate::platform::RelocationSequence as _;
use crate::platform::SectionFlags as _;
use crate::platform::Symbol as _;
use crate::resolution::LoadedMetrics;
use crate::symbol_db::SymbolDb;
use crate::symbol_db::Visibility;
use crate::value_flags::ValueFlags;
use linker_utils::coff::secnames::BSS_SECTION_NAME;
use linker_utils::coff::secnames::DATA_SECTION_NAME;
use linker_utils::coff::secnames::RDATA_SECTION_NAME;
use linker_utils::coff::secnames::TEXT_SECTION_NAME;
use object::LittleEndian;
use object::SectionIndex;
use object::SymbolIndex;
use object::coff::CoffHeader as _;
use object::coff::Symbol as _;
use object::pe::ImageRelocation;
use object::pe::RelocationType;
use object::pe::SectionFlags;
use std::borrow::Cow;
use std::marker::PhantomData;

pub(crate) fn link_for_arch<'data, F: FileSystem>(
    linker: &'data crate::Linker<F>,
    args: &'data CoffArgs,
) -> Result<crate::LinkerOutput<'data>> {
    if !(cfg!(feature = "coff") || args.common().experimental_platforms) {
        bail!(
            "Windows support is still experimental. Rebuild with `--features coff` to enable it."
        );
    }

    match args.architecture() {
        Architecture::X86_64 => linker.link_for_arch::<Coff, crate::coff_x86_64::CoffX86_64>(args),
        _ => {
            todo!()
        }
    }
}

// TODO: Not really a symtab entry right now, we have additional fields to make early iterations
// easier.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SymtabEntry {
    pub(crate) raw: object::pe::ImageSymbol,
    pub(crate) index: SymbolIndex,
    pub(crate) is_aux: bool,
    is_hidden: bool,
}

pub(crate) type DosHeader = object::pe::ImageDosHeader;
pub(crate) type NtHeader = object::pe::ImageNtHeaders64;
pub(crate) type CoffFileHeader = object::pe::ImageFileHeader;
pub(crate) type SectionHeader = object::pe::ImageSectionHeader;

// `object` currently skips directory entries, so the sizes are off. This might change in the
// future.
pub(crate) const SIZE_OF_DIRECTORY_ENTRIES: usize =
    object::pe::IMAGE_NUMBEROF_DIRECTORY_ENTRIES * size_of::<object::pe::ImageDataDirectory>();
const _: () = assert!(SIZE_OF_DIRECTORY_ENTRIES == 128);
pub(crate) const SIZE_OF_OPTIONAL_HEADER64: usize =
    size_of::<object::pe::ImageOptionalHeader64>() + SIZE_OF_DIRECTORY_ENTRIES;
const _: () = assert!(
    SIZE_OF_OPTIONAL_HEADER64
        == size_of::<object::pe::ImageOptionalHeader64>() + SIZE_OF_DIRECTORY_ENTRIES
);
pub(crate) const SIZE_OF_NT_HEADERS64: usize = size_of::<NtHeader>() + SIZE_OF_DIRECTORY_ENTRIES;
const _: () = assert!(
    SIZE_OF_NT_HEADERS64 == size_of::<object::pe::ImageNtHeaders64>() + SIZE_OF_DIRECTORY_ENTRIES
);

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct Coff;

#[repr(u32)]
#[derive(Clone, Copy)]
enum SinglePartSectionId {
    DosStub = crate::output_section_id::NUM_COMMON_SINGLE_PART_SECTIONS,
    NtHeader,
    SectionHeaders,
    Symtab,
    Strtab,

    // Must be last.
    Count,
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum RegularSectionId {
    Text,
    Rdata,
    Data,
    Bss,

    // Must be last.
    Count,
}

const COFF_NUM_SINGLE_PART_SECTIONS: u32 = SinglePartSectionId::Count as u32;
const COFF_NUM_BUILT_IN_REGULAR_SECTIONS: usize = RegularSectionId::Count as usize;
const COFF_NUM_BUILT_IN_SECTIONS: usize =
    COFF_NUM_SINGLE_PART_SECTIONS as usize + COFF_NUM_BUILT_IN_REGULAR_SECTIONS;

pub(crate) mod part_id {
    use super::SinglePartSectionId;
    use crate::part_id::PartId;

    pub(crate) const DOS_STUB: PartId = SinglePartSectionId::DosStub.part_id();
    pub(crate) const NT_HEADER: PartId = SinglePartSectionId::NtHeader.part_id();
    pub(crate) const SECTION_HEADERS: PartId = SinglePartSectionId::SectionHeaders.part_id();
    pub(crate) const SYMTAB: PartId = SinglePartSectionId::Symtab.part_id();
    pub(crate) const STRTAB: PartId = SinglePartSectionId::Strtab.part_id();
}

pub(crate) mod output_section_id {
    use super::RegularSectionId;
    use super::SinglePartSectionId;
    use crate::output_section_id::OutputSectionId;

    pub(crate) const DOS_STUB: OutputSectionId = SinglePartSectionId::DosStub.output_section_id();
    pub(crate) const NT_HEADER: OutputSectionId = SinglePartSectionId::NtHeader.output_section_id();
    pub(crate) const SECTION_HEADERS: OutputSectionId =
        SinglePartSectionId::SectionHeaders.output_section_id();
    pub(crate) const SYMTAB: OutputSectionId = SinglePartSectionId::Symtab.output_section_id();
    pub(crate) const STRTAB: OutputSectionId = SinglePartSectionId::Strtab.output_section_id();

    pub(crate) const TEXT: OutputSectionId = RegularSectionId::Text.output_section_id();
    pub(crate) const RDATA: OutputSectionId = RegularSectionId::Rdata.output_section_id();
    pub(crate) const DATA: OutputSectionId = RegularSectionId::Data.output_section_id();
    pub(crate) const BSS: OutputSectionId = RegularSectionId::Bss.output_section_id();
}

#[derive(derive_more::Debug)]
pub(crate) struct File<'data> {
    pub(crate) arch: Architecture,
    #[debug(skip)]
    pub(crate) data: &'data [u8],
    // `object` follows COFF and uses 1-based section numbers, this naturally conflicts with
    // our 0-based section numbering. Currently we offset section indexes in our methods.
    // TODO: Need to think how to properly solve it.
    pub(crate) sections: object::coff::SectionTable<'data>,
    pub(crate) symbol_table: object::read::coff::SymbolTable<'data>,
    pub(crate) symbols: Vec<SymtabEntry>,
    is_dynamic: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct CoffRelocation {
    raw: ImageRelocation,
}

impl CoffRelocation {
    pub(crate) fn new(raw: ImageRelocation) -> Self {
        Self { raw }
    }
}

impl Relocation for CoffRelocation {
    type Sequence<'data> = RelocationList<'data>;
    type Platform = Coff;

    fn symbol(&self) -> Option<SymbolIndex> {
        Some(self.raw.symbol())
    }

    fn raw_type(&self) -> RelocationType {
        self.raw.typ.get(LittleEndian)
    }

    fn offset(&self) -> u64 {
        self.raw.virtual_address.get(LittleEndian).into()
    }

    // TODO: this is a bit abused to handle bias for some of AMD64 relocations
    fn addend(&self) -> i64 {
        match self.raw_type() {
            object::pe::IMAGE_REL_AMD64_REL32 => -4,
            object::pe::IMAGE_REL_AMD64_REL32_1 => -5,
            object::pe::IMAGE_REL_AMD64_REL32_2 => -6,
            object::pe::IMAGE_REL_AMD64_REL32_3 => -7,
            object::pe::IMAGE_REL_AMD64_REL32_4 => -8,
            object::pe::IMAGE_REL_AMD64_REL32_5 => -9,
            _ => 0,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct RelocationList<'data>(&'data [ImageRelocation]);

impl<'data> platform::RelocationList<'data> for RelocationList<'data> {
    fn num_relocations(&self) -> usize {
        self.0.len()
    }
}

impl<'data> platform::RelocationSequence<'data> for RelocationList<'data> {
    type Rel = CoffRelocation;

    fn rel_iter(&self) -> impl Iterator<Item = CoffRelocation> {
        self.0.iter().copied().map(CoffRelocation::new)
    }

    fn subsequence(&self, range: std::ops::Range<usize>) -> Self {
        Self(&self.0[range])
    }

    fn num_relocations(&self) -> usize {
        self.0.len()
    }
}

// Not needing Drop opens the option of storing this type in an arena that doesn't support dropping
// its contents.
// TODO
// const _: () = assert!(!core::mem::needs_drop::<File>());

impl Platform for Coff {
    const NUM_SINGLE_PART_SECTIONS: u32 = COFF_NUM_SINGLE_PART_SECTIONS;
    const NUM_BUILT_IN_REGULAR_SECTIONS: usize = COFF_NUM_BUILT_IN_REGULAR_SECTIONS;

    const TEXT_SECTION_ID: Option<OutputSectionId> = Some(output_section_id::TEXT);
    const RODATA_SECTION_ID: Option<OutputSectionId> = Some(output_section_id::RDATA);
    const DATA_SECTION_ID: Option<OutputSectionId> = Some(output_section_id::DATA);
    const BSS_SECTION_ID: Option<OutputSectionId> = Some(output_section_id::BSS);
    const STRTAB_SECTION_ID: Option<OutputSectionId> = Some(output_section_id::STRTAB);

    const CUSTOM_PHDR_EXCLUDED_SECTION_IDS: &'static [OutputSectionId] = &[];

    const VERIFY_IGNORE_SECTION_IDS: &'static [OutputSectionId] = &[
        crate::output_section_id::FILE_HEADER,
        output_section_id::DOS_STUB,
        output_section_id::NT_HEADER,
        output_section_id::SECTION_HEADERS,
        output_section_id::SYMTAB,
    ];

    type File<'data> = File<'data>;
    type FileFlags = u32;
    type SymtabEntry = SymtabEntry;
    type PlatformSpecificSymbol = ();
    type SectionHeader = SectionHeader;
    type SectionFlags = SectionFlags;
    type SectionAttributes = SectionAttributes;
    type SectionType = SectionType;
    type SegmentType = ();
    type ProgramSegmentDef = ProgramSegmentDef;
    type BuiltInSectionDetails = BuiltInSectionDetails;
    type RelocationSections = ();
    type DynamicEntry = ();
    type DynamicSymbolDefinitionExt = ();
    type RelocationInfo = RelocationType;
    type NonAddressableIndexes = NonAddressableIndexes;
    type NonAddressableCounts = ();
    type EpilogueLayoutExt = ();
    type GroupLayoutExt = ();
    type CommonGroupStateExt = ();
    type StubLibraryLayoutStateExt = ();
    type StubLibraryLayoutExt = ();
    type ArchIdentifier = ();
    type Args = CoffArgs;
    type ResolutionExt = ();
    type SymtabShndxEntry = ();
    type ResolvedObjectExt<'data> = ();
    type FinaliseSizesExt<'data> = ();
    type GcUnit = SectionGcUnit;
    type SectionIdentityExt = ();
    type SymbolVersionIndex = ();
    type LayoutExt<'data> = ();
    type SectionIterator<'a> = core::slice::Iter<'a, SectionHeader>;
    type DynamicTagValues<'data> = DynamicTagValues;
    type RelocationList<'data> = RelocationList<'data>;
    type DynamicLayoutStateExt<'data> = ();
    type DynamicLayoutExt<'data> = ();
    type LayoutResourcesExt<'data> = ();
    type PreludeLayoutStateExt = ();
    type PreludeLayoutExt = ();
    type ObjectLayoutStateExt<'data> = ObjectLayoutStateExt;
    type RawSymbolName<'data> = RawSymbolName<'data>;
    type VersionNames<'data> = ();
    type VerneedTable<'data> = VerneedTable<'data>;

    fn write_output_file<'data, A: Arch<Platform = Self>, F: FileSystem>(
        output: &crate::file_writer::Output<F>,
        layout: &layout::Layout<'data, Self>,
    ) -> Result {
        output.write(layout, coff_writer::write::<A>)
    }

    fn section_attributes(header: &Self::SectionHeader) -> Self::SectionAttributes {
        Self::SectionAttributes {
            flags: header.characteristics.get(LittleEndian),
        }
    }

    fn apply_force_keep_sections(
        _keep_sections: &mut crate::output_section_map::OutputSectionMap<bool>,
        _args: &Self::Args,
    ) {
    }

    fn is_zero_sized_section_content(_section_id: OutputSectionId) -> bool {
        todo!()
    }

    fn built_in_section_details() -> &'static [Self::BuiltInSectionDetails] {
        &Self::SECTION_DEFINITIONS
    }

    fn finalise_group_layout(_memory_offsets: &OutputSectionPartMap<u64>) -> Self::GroupLayoutExt {}

    fn frame_data_base_address(_memory_offsets: &OutputSectionPartMap<u64>) -> u64 {
        todo!()
    }

    fn align_load_segment_start(
        _segment_def: Self::ProgramSegmentDef,
        _segment_alignment: Alignment,
        _file_offset: &mut usize,
        _mem_offset: &mut u64,
    ) {
    }

    fn activate_dynamic<'data>(
        _state: &mut layout::DynamicLayoutState<'data, Self>,
        _common: &mut layout::CommonGroupState<'data, Self>,
    ) {
    }

    fn pre_finalise_sizes_prelude<'scope, 'data>(
        _prelude: &mut layout::PreludeLayoutState<'data, Self>,
        common: &mut layout::CommonGroupState<'data, Self>,
        resources: &layout::GraphResources<'data, 'scope, Self>,
    ) {
        if !resources.symbol_db.args.should_strip_all() {
            // The first 4 bytes in string table specify its size.
            common.allocate(part_id::STRTAB, size_of::<u32>() as u64);
        }
    }

    fn finalise_sizes_dynamic<'data>(
        _object: &mut layout::DynamicLayoutState<'data, Self>,
        _common: &mut layout::CommonGroupState<'data, Self>,
    ) -> Result {
        Ok(())
    }

    fn finalise_object_sizes<'data>(
        _object: &mut layout::ObjectLayoutState<'data, Self>,
        _common: &mut layout::CommonGroupState<'data, Self>,
    ) {
    }

    fn finalise_object_layout<'data>(
        _object: &layout::ObjectLayoutState<'data, Self>,
        _memory_offsets: &mut OutputSectionPartMap<u64>,
    ) {
    }

    fn finalise_layout_dynamic<'data>(
        _state: &mut layout::DynamicLayoutState<'data, Self>,
        _memory_offsets: &mut OutputSectionPartMap<u64>,
        _resources: &layout::FinaliseLayoutResources<'_, 'data, Self>,
        _resolutions_out: &mut layout::ResolutionWriter<Self>,
    ) -> Result<Option<Self::DynamicLayoutExt<'data>>> {
        todo!()
    }

    fn take_dynsym_index(
        _memory_offsets: &mut OutputSectionPartMap<u64>,
        _section_layouts: &crate::output_section_map::OutputSectionMap<layout::OutputRecordLayout>,
    ) -> Result<u32> {
        todo!()
    }

    fn compute_object_addresses<'data>(
        _object: &layout::ObjectLayoutState<'data, Self>,
        _memory_offsets: &mut OutputSectionPartMap<u64>,
    ) {
    }

    fn layout_resources_ext<'data>(
        _groups: &[crate::grouping::Group<'data, Self>],
    ) -> Self::LayoutResourcesExt<'data> {
    }

    fn gc_unit_for_symbol<'data>(
        object: &Self::File<'data>,
        symbol: &Self::SymtabEntry,
        symbol_index: object::SymbolIndex,
    ) -> Result<Option<Self::GcUnit>> {
        Ok(object
            .symbol_section(symbol, symbol_index)?
            .map(SectionGcUnit::new))
    }

    fn activate_object_gc<'data, 'scope, A: Arch<Platform = Self>>(
        object: &mut layout::ObjectLayoutState<'data, Self>,
        common: &mut layout::CommonGroupState<'data, Self>,
        resources: &'scope layout::GraphResources<'data, 'scope, Self>,
        queue: &mut layout::LocalWorkQueue<Self>,
        scope: &rayon::Scope<'scope>,
    ) -> Result {
        object.activate_section_gc::<A>(common, resources, queue, scope)
    }

    fn load_gc_unit<'data, 'scope, A: Arch<Platform = Self>>(
        object: &mut layout::ObjectLayoutState<'data, Self>,
        common: &mut layout::CommonGroupState<'data, Self>,
        resources: &'scope layout::GraphResources<'data, 'scope, Self>,
        queue: &mut layout::LocalWorkQueue<Self>,
        unit: Self::GcUnit,
        scope: &rayon::Scope<'scope>,
    ) -> Result {
        object.handle_section_load_request::<A>(
            common,
            resources,
            queue,
            unit.section_index(),
            scope,
        )
    }

    fn load_object_section_relocations<'data, 'scope, A: Arch<Platform = Self>>(
        state: &mut layout::ObjectLayoutState<'data, Self>,
        _common: &mut layout::CommonGroupState<'data, Self>,
        queue: &mut layout::LocalWorkQueue<Self>,
        resources: &'scope layout::GraphResources<'data, '_, Self>,
        _section: layout::Section,
        section_index: SectionIndex,
        scope: &rayon::Scope<'scope>,
    ) -> Result {
        for relocation in state.relocations(section_index)?.rel_iter() {
            let local_symbol_id = state
                .symbol_id_range
                .input_to_id(relocation.symbol().unwrap());
            let symbol_id = resources.symbol_db.definition(local_symbol_id);

            let atomic_flags = &resources.per_symbol_flags.get_atomic(symbol_id);
            let previous_flags = atomic_flags.fetch_or(ValueFlags::DIRECT);

            if !previous_flags.has_resolution() {
                queue.send_symbol_request::<A>(symbol_id, resources, scope);
            }
        }
        Ok(())
    }

    fn create_dynamic_symbol_definition<'data>(
        _symbol_db: &SymbolDb<'data, Self>,
        _symbol_id: crate::symbol_db::SymbolId,
    ) -> Result<layout::DynamicSymbolDefinition<'data, Self>> {
        todo!()
    }

    fn update_segment_keep_list(
        _program_segments: &crate::program_segments::ProgramSegments<Self::ProgramSegmentDef>,
        _keep_segments: &mut [bool],
        _args: &Self::Args,
    ) {
    }

    fn program_segment_defs() -> &'static [Self::ProgramSegmentDef] {
        &[ProgramSegmentDef {}]
    }

    fn unconditional_segment_defs() -> &'static [Self::ProgramSegmentDef] {
        &[]
    }

    fn program_segment_should_include_section(
        _segment_def: Self::ProgramSegmentDef,
        _section_info: &SectionOutputInfo<Self>,
        _section_id: OutputSectionId,
        _rosegment: bool,
    ) -> bool {
        true
    }

    fn create_linker_defined_symbols(
        symbols: &mut crate::parsing::InternalSymbolsBuilder<Self>,
        _output_kind: crate::output_kind::OutputKind,
        _args: &Self::Args,
        _should_emit_linker_symbols: bool,
    ) {
        // The undefined symbol must always be symbol 0.
        symbols
            .add_symbol(InternalSymDefInfo::new(SymbolPlacement::Undefined, b""))
            .hide();
    }

    fn built_in_section_infos<'data>() -> Vec<SectionOutputInfo<'data, Self>> {
        Self::SECTION_DEFINITIONS
            .iter()
            .map(|d| SectionOutputInfo {
                section_attributes: SectionAttributes {
                    flags: d.section_flags,
                },
                kind: d.kind,
                min_alignment: d.min_alignment,
                location_info: None,
                secondary_order: None,
                region_name: None,
                fill: None,
                phdrs: Vec::new(),
            })
            .collect()
    }

    fn create_finalise_sizes_ext<'data, 'states, 'files, A: Arch<Platform = Self>>(
        _args: &Self::Args,
        _groups: &'files mut [layout::GroupState<'data, Self>],
        _symbol_db: &SymbolDb<'data, Self>,
    ) -> Result<Self::FinaliseSizesExt<'data>>
    where
        'data: 'files,
        'data: 'states,
    {
        Ok(())
    }

    fn create_layout_ext<'data>(
        _finalise_sizes_ext: Self::FinaliseSizesExt<'data>,
        _resolutions: &layout::SymbolResolutions<Self>,
        _group_layouts: &[layout::GroupLayout<'data, Self>],
    ) -> Result<Self::LayoutExt<'data>> {
        Ok(())
    }

    fn load_exception_frame_data<'data, 'scope, A: Arch<Platform = Self>>(
        _object: &mut layout::ObjectLayoutState<'data, Self>,
        _common: &mut layout::CommonGroupState<'data, Self>,
        _eh_frame_section_index: SectionIndex,
        _resources: &'scope layout::GraphResources<'data, '_, Self>,
        _queue: &mut layout::LocalWorkQueue<Self>,
        _scope: &rayon::Scope<'scope>,
    ) -> Result {
        todo!()
    }

    fn non_empty_section_loaded<'data, 'scope, A: Arch<Platform = Self>>(
        _object: &mut layout::ObjectLayoutState<'data, Self>,
        _common: &mut layout::CommonGroupState<'data, Self>,
        _queue: &mut layout::LocalWorkQueue<Self>,
        _unloaded: crate::resolution::UnloadedSection,
        _resources: &'scope layout::GraphResources<'data, 'scope, Self>,
        _scope: &rayon::Scope<'scope>,
    ) -> Result {
        Ok(())
    }

    fn new_epilogue_layout<'data>(
        _args: &Self::Args,
        _output_kind: crate::output_kind::OutputKind,
        _dynamic_symbol_definitions: &mut [layout::DynamicSymbolDefinition<'data, Self>],
        _group_states: &[layout::GroupState<'data, Self>],
    ) -> Self::EpilogueLayoutExt {
    }

    fn apply_non_addressable_indexes_epilogue(
        _counts: &mut Self::NonAddressableCounts,
        _state: &mut Self::EpilogueLayoutExt,
    ) {
    }

    fn apply_non_addressable_indexes<'data, 'groups>(
        _symbol_db: &SymbolDb<'data, Self>,
        _counts: &Self::NonAddressableCounts,
        _mem_sizes_iter: impl Iterator<Item = &'groups mut OutputSectionPartMap<u64>>,
    ) {
    }

    fn finalise_sizes_epilogue<'data>(
        _state: &mut Self::EpilogueLayoutExt,
        _mem_sizes: &mut OutputSectionPartMap<u64>,
        _dynamic_symbol_definitions: &[layout::DynamicSymbolDefinition<'data, Self>],
        _properties: &Self::LayoutExt<'data>,
        _symbol_db: &SymbolDb<'data, Self>,
    ) -> Result<()> {
        Ok(())
    }

    fn finalise_sizes_all<'data>(
        _mem_sizes: &mut OutputSectionPartMap<u64>,
        _symbol_db: &SymbolDb<'data, Self>,
    ) {
    }

    fn finalise_layout_epilogue<'data>(
        _epilogue_state: &mut Self::EpilogueLayoutExt,
        _memory_offsets: &mut OutputSectionPartMap<u64>,
        _symbol_db: &SymbolDb<'data, Self>,
        _common_state: &Self::LayoutExt<'data>,
        _dynsym_start_index: u32,
        _dynamic_symbol_defs: &[layout::DynamicSymbolDefinition<Self>],
    ) -> Result {
        Ok(())
    }

    fn is_symbol_non_interposable<'data>(
        _object: &Self::File<'data>,
        _args: &Self::Args,
        _sym: &Self::SymtabEntry,
        _output_kind: crate::output_kind::OutputKind,
        _export_list: Option<&crate::export_list::ExportList>,
        _lib_name: &[u8],
        _archive_semantics: bool,
        _is_undefined: bool,
    ) -> bool {
        true
    }

    fn allocate_header_sizes<'data>(
        _prelude: &mut layout::PreludeLayoutState<'data, Self>,
        sizes: &mut OutputSectionPartMap<u64>,
        header_info: &layout::HeaderInfo,
        _program_segments: &crate::program_segments::ProgramSegments<Self::ProgramSegmentDef>,
        _output_sections: &crate::output_section_id::OutputSections<Self>,
        _resources: &layout::FinaliseSizesResources<'data, '_, Self>,
        _args: &Self::Args,
    ) {
        sizes.increment(crate::part_id::FILE_HEADER, size_of::<DosHeader>() as u64);
        sizes.increment(part_id::DOS_STUB, DOS_STUB.len() as u64);
        sizes.increment(part_id::NT_HEADER, SIZE_OF_NT_HEADERS64 as u64);
        sizes.increment(part_id::SECTION_HEADERS, section_headers_size(header_info));
    }

    fn finalise_sizes_for_symbol<'data>(
        _common: &mut layout::CommonGroupState<'data, Self>,
        _symbol_db: &SymbolDb<'data, Self>,
        _symbol_id: crate::symbol_db::SymbolId,
        _flags: crate::value_flags::ValueFlags,
    ) -> Result {
        Ok(())
    }

    fn allocate_resolution(
        _flags: crate::value_flags::ValueFlags,
        _mem_sizes: &mut OutputSectionPartMap<u64>,
        _output_kind: crate::output_kind::OutputKind,
        _args: &Self::Args,
        _is_linker_script_symbol: bool,
    ) {
    }

    fn allocate_object_symtab_space<'data>(
        state: &layout::ObjectLayoutState<'data, Self>,
        common: &mut layout::CommonGroupState<'data, Self>,
        symbol_db: &SymbolDb<'data, Self>,
        per_symbol_flags: &crate::value_flags::AtomicPerSymbolFlags,
    ) -> Result {
        let mut num_sym = 0;
        let mut strings_size = 0;
        for ((sym_index, sym), flags) in state
            .object
            .enumerate_symbols()
            .zip(per_symbol_flags.range(state.symbol_id_range()))
            .filter(|((_, sym), _)| !sym.is_aux)
        {
            let symbol_id = state.symbol_id_range.input_to_id(sym_index);
            if let Some(info) = SymbolCopyInfo::new(
                state.object,
                sym_index,
                sym,
                symbol_id,
                symbol_db,
                flags.get(),
                &state.sections,
            ) {
                // TODO: check if applicable
                // If we've decided to emit the symbol even though it's not referenced (because it's
                // in a section we're emitting), then make sure we have a resolution for it.
                flags.fetch_or(ValueFlags::DIRECT);
                num_sym += 1;
                if info.name.len() > 8 {
                    strings_size += info.name.len() + 1;
                }
            }
        }
        let entry_size = object::pe::IMAGE_SIZEOF_SYMBOL as u64;
        common.allocate(part_id::SYMTAB, num_sym * entry_size);
        common.allocate(part_id::STRTAB, strings_size as u64);
        Ok(())
    }

    fn allocate_internal_symbol(
        symbol_id: crate::symbol_db::SymbolId,
        def_info: &InternalSymDefInfo<Self>,
        sizes: &mut OutputSectionPartMap<u64>,
        symbol_db: &SymbolDb<Self>,
    ) -> Result {
        if def_info.symbol.is_hidden() {
            return Ok(());
        }
        let symtab_part = part_id::SYMTAB;
        sizes.increment(symtab_part, object::pe::IMAGE_SIZEOF_SYMBOL as u64);
        let symbol_name = symbol_db.symbol_name(symbol_id)?;
        let symbol_name = symbol_name.bytes();
        if symbol_name.len() > 8 {
            sizes.increment(part_id::STRTAB, symbol_name.len() as u64 + 1);
        }

        Ok(())
    }

    fn allocate_prelude(_common: &mut layout::CommonGroupState<Self>, _symbol_db: &SymbolDb<Self>) {
    }

    fn finalise_prelude_layout<'data>(
        _prelude: &layout::PreludeLayoutState<Self>,
        _memory_offsets: &mut OutputSectionPartMap<u64>,
        _resources: &layout::FinaliseLayoutResources<'_, 'data, Self>,
    ) -> Result<Self::PreludeLayoutExt> {
        Ok(())
    }

    fn create_resolution(
        flags: crate::value_flags::ValueFlags,
        raw_value: u64,
        dynamic_symbol_index: Option<std::num::NonZeroU32>,
        _memory_offsets: &mut OutputSectionPartMap<u64>,
        _args: &<Self as Platform>::Args,
        _output_kind: crate::OutputKind,
        _is_linker_script_symbol: bool,
    ) -> layout::Resolution<Self> {
        layout::Resolution {
            raw_value,
            dynamic_symbol_index,
            flags,
            format_specific: (),
        }
    }

    fn raw_symbol_name<'data>(
        name_bytes: &'data [u8],
        _verneed_table: &Self::VerneedTable<'data>,
        _symbol_index: SymbolIndex,
    ) -> Self::RawSymbolName<'data> {
        RawSymbolName::parse(name_bytes)
    }

    fn default_layout_rules(_args: &Self::Args) -> Vec<SectionRule<'static>> {
        let mut rules = Vec::with_capacity(
            DEFAULT_SECTION_PLACEMENT_RULES.len() + LINKER_MANAGED_SECTION_RULES.len() + 1,
        );

        rules.extend(DEFAULT_SECTION_PLACEMENT_RULES.iter().cloned());
        rules.extend(LINKER_MANAGED_SECTION_RULES.iter().cloned());

        rules
    }

    fn build_output_order_and_program_segments<'data>(
        custom: &crate::output_section_id::CustomSectionIds,
        output_kind: crate::output_kind::OutputKind,
        output_sections: &crate::output_section_id::OutputSections<'data, Self>,
        secondary: &crate::output_section_map::OutputSectionMap<Vec<OutputSectionId>>,
        location_counters: &[layout_rules::LocationCounter<'data>],
    ) -> (
        crate::output_section_id::OutputOrder<'data>,
        crate::program_segments::ProgramSegments<Self::ProgramSegmentDef>,
    ) {
        let mut builder = OutputOrderBuilder::<Self>::new(
            Self::program_segment_defs().to_vec(),
            output_kind,
            output_sections,
            secondary,
            false,
            location_counters,
        );

        builder.add_section(crate::output_section_id::FILE_HEADER);
        builder.add_section(output_section_id::DOS_STUB);
        builder.add_section(output_section_id::NT_HEADER);
        builder.add_section(output_section_id::SECTION_HEADERS);

        builder.add_section(output_section_id::TEXT);
        builder.add_sections(&custom.exec);

        builder.add_section(output_section_id::RDATA);
        builder.add_sections(&custom.ro);
        builder.add_section(output_section_id::DATA);
        builder.add_sections(&custom.data);
        builder.add_section(output_section_id::BSS);
        builder.add_sections(&custom.bss);
        builder.add_sections(&custom.nonalloc);
        builder.add_sections(&custom.tbss);
        builder.add_sections(&custom.tdata);

        builder.add_section(output_section_id::SYMTAB);
        builder.add_section(output_section_id::STRTAB);

        builder.build()
    }

    fn default_symtab_entry() -> Self::SymtabEntry {
        // TODO: Might need to tinker the defaults
        SymtabEntry {
            raw: object::pe::ImageSymbol {
                name: [0; 8],
                value: Default::default(),
                section_number: Default::default(),
                typ: Default::default(),
                storage_class: Default::default(),
                number_of_aux_symbols: 0,
            },
            index: SymbolIndex(0),
            is_aux: false,
            is_hidden: false,
        }
    }

    type GdbIndexScanResult<'data> = ();

    fn new_resolved_object_ext<'data>(
        _symbol_id_range: crate::symbol_db::SymbolIdRange,
        _file_id: crate::input_data::FileId,
    ) -> Self::ResolvedObjectExt<'data> {
    }

    fn new_object_layout_state_ext<'data>(
        _input: Self::ResolvedObjectExt<'data>,
    ) -> Self::ObjectLayoutStateExt<'data> {
        Self::ObjectLayoutStateExt::default()
    }

    fn is_allowed_in_archive(kind: crate::file_kind::FileKind) -> bool {
        kind == FileKind::CoffObject
    }

    fn section_identity<'data>(
        _file: &Self::File<'data>,
        name: SectionName<'data>,
        _section: &Self::SectionHeader,
    ) -> SectionIdentity<'data, Self> {
        Self::section_identity_from_name(name).unwrap()
    }

    fn section_identity_from_name(name: SectionName) -> Option<SectionIdentity<Self>> {
        Some(SectionIdentity::new(name, ()))
    }
}

impl<'data> File<'data> {
    fn parse_coff_bytes(data: &'data [u8], is_dynamic: bool) -> Result<Self> {
        let mut offset = 0;
        let header = CoffFileHeader::parse(data, &mut offset)?;
        let arch = match header.machine() {
            object::pe::IMAGE_FILE_MACHINE_AMD64 => Architecture::X86_64,
            _ => bail!("Unsupported architecture: {:?}", header.machine()),
        };

        let sections = header.sections(data, offset)?;

        // This is a bit of a hack.
        // Wild needs dense symbol list, but COFF stores symbols along with zero or more of their
        // aux symbols.
        //
        // `object` crate handles that by densely storing symbols and their aux records, and
        // reflects that in indexes. However, its iterators return only the "real" symbols.
        //
        // This causes Wild's own indexing to fall apart. To fix that we can either:
        // - store symbols and aux records a separate symbols, to retain `object`'s indexing
        // - store symbols only and create map between to convert between our and `object`'s
        //   indexing
        // - avoid using `object`'s parsing and do it ourselves
        //
        // The code below implements the first option. This also gives us a bit of flexibility for
        // example to nicely handle hidden symbols.
        let symbol_table = header.symbols(data)?;
        let num_symbols = symbol_table.len();
        let mut symbols = Vec::with_capacity(num_symbols);
        let mut aux_remaining = 0;
        for i in 0..num_symbols {
            let raw = *symbol_table.symbol(SymbolIndex(i))?;
            let index = SymbolIndex(i);
            if aux_remaining > 0 {
                aux_remaining -= 1;
                symbols.push(SymtabEntry {
                    raw,
                    index,
                    is_aux: true,
                    is_hidden: false,
                });
            } else {
                aux_remaining = raw.number_of_aux_symbols;
                symbols.push(SymtabEntry {
                    raw,
                    index,
                    is_aux: false,
                    is_hidden: false,
                });
            }
        }

        Ok(Self {
            arch,
            data,
            sections,
            symbol_table,
            symbols,
            is_dynamic,
        })
    }
}

impl<'data> platform::ObjectFile<'data> for File<'data> {
    type Platform = Coff;

    fn parse_bytes(data: &'data [u8], is_dynamic: bool) -> Result<Self> {
        Self::parse_coff_bytes(data, is_dynamic)
    }

    fn parse(input: &InputBytes<'data>, args: &CoffArgs) -> Result<Self> {
        let file = Self::parse_bytes(input.data, false)?;

        if file.arch != args.architecture() {
            bail!(
                "`{}` has incompatible architecture: {}, expecting {}",
                input,
                file.arch,
                args.architecture(),
            );
        }

        Ok(file)
    }

    fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    fn num_symbols(&self) -> usize {
        self.symbols.len()
    }

    fn symbols_iter(&self) -> impl Iterator<Item = &SymtabEntry> {
        self.symbols.iter()
    }

    fn symbol(&self, index: SymbolIndex) -> Result<&SymtabEntry> {
        self.symbols
            .get(index.0)
            .ok_or_else(|| crate::error!("Invalid symbol index {index:?}"))
    }

    fn section_size(&self, section: &SectionHeader) -> Result<u64> {
        let size = u64::from(section.virtual_size.get(LittleEndian));
        if size == 0 {
            return Ok(u64::from(section.size_of_raw_data.get(LittleEndian)));
        }
        Ok(size)
    }

    fn symbol_name(&self, symbol: &SymtabEntry) -> Result<&'data [u8]> {
        if symbol.is_aux {
            bail!("aux symbol has no name")
        }
        // TODO: symbol lookup could be avoided by adding `'data` lifetime to `SymtabEntry`
        let sym = self.symbol_table.symbol(symbol.index)?;
        Ok(sym.name(self.symbol_table.strings())?)
    }

    fn symbol_offset_in_section(
        &self,
        symbol: &SymtabEntry,
        _section_index: SectionIndex,
    ) -> Result<u64> {
        Ok(symbol.value())
    }

    fn num_sections(&self) -> usize {
        self.sections.len()
    }

    fn section_iter(&self) -> core::slice::Iter<'_, SectionHeader> {
        self.sections.iter()
    }

    fn enumerate_sections(&self) -> impl Iterator<Item = (SectionIndex, &SectionHeader)> {
        self.sections
            .iter()
            .enumerate()
            .map(|(i, section)| (SectionIndex(i), section))
    }

    fn section(&self, index: SectionIndex) -> Result<&SectionHeader> {
        Ok(self.sections.section(SectionIndex(index.0 + 1))?)
    }

    fn section_by_name(&self, name: &str) -> Option<(SectionIndex, &SectionHeader)> {
        self.sections
            .section_by_name(self.symbol_table.strings(), name.as_bytes())
            .map(|(i, section)| (SectionIndex(i.0 - 1), section))
    }

    fn symbol_section(
        &self,
        symbol: &SymtabEntry,
        _index: SymbolIndex,
    ) -> Result<Option<SectionIndex>> {
        if symbol.is_aux {
            return Ok(None);
        }
        Ok(symbol.raw.section().map(|i| SectionIndex(i.0 - 1)))
    }

    fn symbol_versions(&self) -> &[()] {
        &[]
    }

    fn finalise_sizes_dynamic(
        &self,
        _lib_name: &[u8],
        _state: &mut (),
        _mem_sizes: &mut OutputSectionPartMap<u64>,
    ) -> Result {
        todo!()
    }

    fn apply_non_addressable_indexes_dynamic(
        &self,
        _indexes: &mut NonAddressableIndexes,
        _counts: &mut (),
        _state: &mut (),
    ) -> Result {
        todo!()
    }

    fn section_name(&self, index: SectionIndex) -> Result<&'data [u8]> {
        let section = self.sections.section(SectionIndex(index.0 + 1))?;
        Ok(section.name(self.symbol_table.strings())?)
    }

    fn raw_section_data(&self, section: &SectionHeader) -> Result<&'data [u8]> {
        let e = LittleEndian;
        let ptr = section.pointer_to_raw_data.get(e) as usize;
        let mut size = section.size_of_raw_data.get(e) as usize;
        if size == 0 {
            size = section.virtual_size.get(e) as usize;
        }
        Ok(&self.data[ptr..ptr + size])
    }

    fn section_data(
        &self,
        section: &SectionHeader,
        _member: &bumpalo_herd::Member<'data>,
        _loaded_metrics: &LoadedMetrics,
    ) -> Result<&'data [u8]> {
        self.raw_section_data(section)
    }

    fn copy_section_data(&self, section: &SectionHeader, out: &mut [u8]) -> Result {
        let bytes = self.raw_section_data(section)?;
        out.copy_from_slice(bytes);
        Ok(())
    }

    fn section_data_cow(&self, section: &SectionHeader) -> Result<Cow<'data, [u8]>> {
        Ok(Cow::Borrowed(self.raw_section_data(section)?))
    }

    fn section_alignment(&self, section: &SectionHeader) -> Result<u64> {
        Ok(section.coff_alignment())
    }

    fn relocations(&self, index: SectionIndex, _relocations: &()) -> Result<RelocationList<'data>> {
        let section = self.section(index)?;
        let raw = section.coff_relocations(self.data)?;
        Ok(RelocationList(raw))
    }

    fn parse_relocations(&self) -> Result<()> {
        // TODO: Just so we can forward right now
        Ok(())
    }

    fn symbol_version_debug(&self, _symbol_index: SymbolIndex) -> Option<String> {
        None
    }

    fn section_display_name(&self, index: SectionIndex) -> Cow<'data, str> {
        self.section_name(index).map_or_else(
            |_| format!("<index {}>", index.0).into(),
            String::from_utf8_lossy,
        )
    }

    fn dynamic_tag_values(&self) -> Option<DynamicTagValues> {
        None
    }

    fn get_version_names(&self) -> Result<VersionNames> {
        Ok(())
    }

    fn get_symbol_name_and_version(
        &self,
        _symbol: &SymtabEntry,
        _local_index: usize,
        _version_names: &VersionNames,
    ) -> Result<RawSymbolName<'data>> {
        todo!()
    }

    fn should_enforce_undefined(
        &self,
        _resources: &layout::GraphResources<'data, '_, Coff>,
    ) -> bool {
        todo!()
    }

    fn verneed_table(&self) -> Result<VerneedTable<'data>> {
        Ok(VerneedTable::default())
    }

    fn process_gnu_note_section(
        &self,
        _state: &mut ObjectLayoutStateExt,
        _section_index: SectionIndex,
    ) -> Result {
        todo!()
    }

    fn dynamic_tags(&self) -> Result<&'data [()]> {
        Ok(&[])
    }
}

impl platform::SectionHeader for SectionHeader {
    fn is_alloc(&self) -> bool {
        self.characteristics.get(LittleEndian).is_alloc()
    }

    fn is_writable(&self) -> bool {
        self.characteristics
            .get(LittleEndian)
            .contains(object::pe::IMAGE_SCN_MEM_WRITE)
    }

    fn is_executable(&self) -> bool {
        self.characteristics
            .get(LittleEndian)
            .contains(object::pe::IMAGE_SCN_MEM_EXECUTE)
    }

    fn is_tls(&self) -> bool {
        todo!()
    }

    fn is_merge_section(&self) -> bool {
        false
    }

    fn is_strings(&self) -> bool {
        false
    }

    fn should_retain(&self) -> bool {
        false
    }

    fn should_exclude(&self) -> bool {
        self.characteristics
            .get(LittleEndian)
            .contains(object::pe::IMAGE_SCN_LNK_REMOVE)
    }

    fn is_group(&self) -> bool {
        false
    }

    fn is_note(&self) -> bool {
        false
    }

    fn is_prog_bits(&self) -> bool {
        self.characteristics
            .get(LittleEndian)
            .contains(object::pe::IMAGE_SCN_CNT_INITIALIZED_DATA)
    }

    fn is_no_bits(&self) -> bool {
        self.characteristics
            .get(LittleEndian)
            .contains(object::pe::IMAGE_SCN_CNT_UNINITIALIZED_DATA)
    }

    fn is_null(&self) -> bool {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct SectionType {}

impl platform::SectionType for SectionType {
    fn is_rela(&self) -> bool {
        todo!()
    }

    fn is_rel(&self) -> bool {
        todo!()
    }

    fn is_symtab(&self) -> bool {
        todo!()
    }

    fn is_strtab(&self) -> bool {
        todo!()
    }
}

impl platform::SectionFlags for SectionFlags {
    fn is_alloc(self) -> bool {
        self.contains(object::pe::IMAGE_SCN_MEM_READ)
    }
}

impl platform::Symbol for SymtabEntry {
    fn as_common(&self) -> Option<CommonSymbol> {
        // TODO
        None
    }

    fn is_undefined(&self) -> bool {
        !self.is_aux
            && self.raw.section_number() == object::pe::IMAGE_SYM_UNDEFINED
            && self.raw.value() == 0
    }

    fn is_local(&self) -> bool {
        self.is_aux
            // TODO: this is basically object::read::coff::symbol::CoffSymbol::is_global
            || !matches!(
                self.raw.storage_class(),
                object::pe::IMAGE_SYM_CLASS_EXTERNAL
                    | object::pe::IMAGE_SYM_CLASS_WEAK_EXTERNAL
            )
    }

    fn is_absolute(&self) -> bool {
        self.is_aux || self.raw.section_number() == object::pe::IMAGE_SYM_ABSOLUTE
    }

    fn is_weak(&self) -> bool {
        !self.is_aux && self.raw.storage_class == object::pe::IMAGE_SYM_CLASS_WEAK_EXTERNAL
    }

    fn visibility(&self) -> Visibility {
        Visibility::Default
    }

    fn value(&self) -> u64 {
        self.raw.value().into()
    }

    fn size(&self) -> u64 {
        debug_assert!(self.is_common());
        self.value()
    }

    fn has_name(&self) -> bool {
        todo!()
    }

    fn is_default_strippable(&self, _name: &[u8]) -> bool {
        // LLD skips symbols with file and section name. We follow that to make our life easier.
        self.is_aux || self.raw.has_aux_file_name() || self.raw.has_aux_section()
    }

    fn debug_string(&self) -> String {
        SymDebug(self).to_string()
    }

    fn is_tls(&self) -> bool {
        todo!()
    }

    fn is_interposable(&self) -> bool {
        false
    }

    fn is_func(&self) -> bool {
        todo!()
    }

    fn is_ifunc(&self) -> bool {
        false
    }

    fn is_hidden(&self) -> bool {
        self.is_hidden
    }

    fn is_gnu_unique(&self) -> bool {
        false
    }

    fn with_hidden(mut self, hidden: bool) -> Self {
        self.is_hidden = hidden;
        self
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct DynamicTagValues {}

impl<'data> platform::DynamicTagValues<'data> for DynamicTagValues {
    fn lib_name(&self, _input: &InputRef<'data>) -> &'data [u8] {
        todo!()
    }
}

struct SymDebug<'data>(pub(crate) &'data SymtabEntry);

impl std::fmt::Display for SymDebug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sym = self.0;

        // TODO: Make it more sensible for COFF
        let vis = if sym.is_local() {
            "Local"
        } else if sym.is_weak() {
            "Weak"
        } else {
            "Global"
        };

        let kind = if sym.is_undefined() {
            "Undefined"
        } else {
            match sym.raw.storage_class {
                object::pe::IMAGE_SYM_CLASS_EXTERNAL => "External",
                object::pe::IMAGE_SYM_CLASS_STATIC => "Static",
                object::pe::IMAGE_SYM_CLASS_EXTERNAL_DEF => "ExternalDef",
                object::pe::IMAGE_SYM_CLASS_WEAK_EXTERNAL => "WeakExternal",
                _ => todo!(),
            }
        };

        write!(f, "{vis} {kind}")
    }
}

#[derive(Default)]
pub(crate) struct ObjectLayoutStateExt {}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct SectionAttributes {
    pub(crate) flags: SectionFlags,
}

impl platform::SectionAttributes for SectionAttributes {
    type Platform = Coff;

    fn merge(&mut self, rhs: Self) {
        self.flags |= rhs.flags;
    }

    fn apply(
        &self,
        output_sections: &mut crate::output_section_id::OutputSections<Self::Platform>,
        section_id: OutputSectionId,
    ) {
        let info = output_sections.section_infos.get_mut(section_id);

        info.section_attributes.flags |= self.flags;
    }

    fn is_null(&self) -> bool {
        false
    }

    fn is_alloc(&self) -> bool {
        self.flags.is_alloc()
    }

    fn is_executable(&self) -> bool {
        self.flags.contains(object::pe::IMAGE_SCN_MEM_EXECUTE)
    }

    fn is_tls(&self) -> bool {
        false
    }

    fn occupies_only_tls_address_space(&self) -> bool {
        false
    }

    fn is_writable(&self) -> bool {
        self.flags.contains(object::pe::IMAGE_SCN_MEM_WRITE)
    }

    fn is_no_bits(&self) -> bool {
        self.flags
            .contains(object::pe::IMAGE_SCN_CNT_UNINITIALIZED_DATA)
    }

    fn flags(&self) -> <Self::Platform as Platform>::SectionFlags {
        self.flags
    }

    fn ty(&self) -> <Self::Platform as Platform>::SectionType {
        SectionType::default()
    }

    fn set_to_default_type(&mut self) {}
}

pub(crate) type VersionNames = ();

pub(crate) struct RawSymbolName<'data> {
    name: &'data [u8],
}

impl<'data> platform::RawSymbolName<'data> for RawSymbolName<'data> {
    fn parse(bytes: &'data [u8]) -> Self {
        Self { name: bytes }
    }

    fn name(&self) -> &'data [u8] {
        self.name
    }

    fn version_name(&self) -> Option<&'data [u8]> {
        None
    }

    fn is_default(&self) -> bool {
        true
    }
}

impl std::fmt::Display for RawSymbolName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.name))?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct VerneedTable<'data> {
    _phantom: PhantomData<&'data ()>,
}

impl<'data> platform::VerneedTable<'data> for VerneedTable<'data> {
    fn version_name(&self, _local_symbol_index: SymbolIndex) -> Option<&'data [u8]> {
        None
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct NonAddressableIndexes {}

impl platform::NonAddressableIndexes for NonAddressableIndexes {
    fn new<P: Platform>(_symbol_db: &SymbolDb<P>) -> Self {
        Self::default()
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct ProgramSegmentDef {}

impl platform::ProgramSegmentDef for ProgramSegmentDef {
    fn is_writable(self) -> bool {
        todo!()
    }

    fn is_executable(self) -> bool {
        todo!()
    }

    fn always_keep(self) -> bool {
        false
    }

    fn is_loadable(self) -> bool {
        true
    }

    fn is_stack(self) -> bool {
        false
    }

    fn is_tls(self) -> bool {
        false
    }

    fn order_key(self) -> usize {
        todo!()
    }
}

impl std::fmt::Display for ProgramSegmentDef {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct BuiltInSectionDetails {
    pub(crate) kind: SectionKind<'static, Coff>,
    pub(crate) section_flags: SectionFlags,
    pub(crate) min_alignment: Alignment,
}

impl Coff {
    const DEFAULT_DEFS: BuiltInSectionDetails = BuiltInSectionDetails {
        kind: Self::primary_section(&[]),
        section_flags: object::pe::IMAGE_SCN_MEM_READ,
        min_alignment: alignment::PE_SECTION_ALIGNMENT,
    };
    const DEFAULT_HEADER_DEFS: BuiltInSectionDetails = BuiltInSectionDetails {
        kind: Self::primary_section(&[]),
        section_flags: object::pe::IMAGE_SCN_MEM_READ.with(object::pe::IMAGE_SCN_LNK_REMOVE),
        min_alignment: alignment::MIN,
    };

    const fn primary_section(name: &'static [u8]) -> SectionKind<'static, Coff> {
        SectionKind::Primary(SectionIdentity::new(SectionName(name), ()))
    }

    pub(crate) const SECTION_DEFINITIONS: [BuiltInSectionDetails; COFF_NUM_BUILT_IN_SECTIONS] = {
        let mut defs = [Self::DEFAULT_DEFS; COFF_NUM_BUILT_IN_SECTIONS];

        defs[crate::output_section_id::FILE_HEADER.as_usize()] = Self::DEFAULT_HEADER_DEFS;
        defs[output_section_id::DOS_STUB.as_usize()] = Self::DEFAULT_HEADER_DEFS;
        defs[output_section_id::NT_HEADER.as_usize()] = Self::DEFAULT_HEADER_DEFS;
        defs[output_section_id::SECTION_HEADERS.as_usize()] = Self::DEFAULT_HEADER_DEFS;
        defs[output_section_id::SYMTAB.as_usize()] = Self::DEFAULT_HEADER_DEFS;
        defs[output_section_id::STRTAB.as_usize()] = Self::DEFAULT_HEADER_DEFS;

        // Start of regular sections
        defs[output_section_id::TEXT.as_usize()] = BuiltInSectionDetails {
            kind: Self::primary_section(TEXT_SECTION_NAME),
            ..Self::DEFAULT_DEFS
        };
        defs[output_section_id::RDATA.as_usize()] = BuiltInSectionDetails {
            kind: Self::primary_section(RDATA_SECTION_NAME),
            ..Self::DEFAULT_DEFS
        };
        defs[output_section_id::DATA.as_usize()] = BuiltInSectionDetails {
            kind: Self::primary_section(DATA_SECTION_NAME),
            ..Self::DEFAULT_DEFS
        };
        defs[output_section_id::BSS.as_usize()] = BuiltInSectionDetails {
            kind: Self::primary_section(BSS_SECTION_NAME),
            ..Self::DEFAULT_DEFS
        };

        defs
    };
}

impl platform::BuiltInSectionDetails for BuiltInSectionDetails {}

impl Resolution<Coff> {
    #[inline(always)]
    pub(crate) fn value_with_addend(&self, addend: i64) -> u64 {
        // TODO: ELF has some optimisation that while unlikely, might apply here.
        self.raw_value.wrapping_add(addend as u64)
    }
}

// TODO: Most of them could use exact match, but that needs to be verified.
const DEFAULT_SECTION_PLACEMENT_RULES: &[SectionRule<'static>] = &[
    SectionRule::prefix_section(".text".as_bytes(), output_section_id::TEXT),
    SectionRule::prefix_section(".rdata".as_bytes(), output_section_id::RDATA),
    SectionRule::prefix_section(".data".as_bytes(), output_section_id::DATA),
    SectionRule::prefix_section(".bss".as_bytes(), output_section_id::BSS),
];

const LINKER_MANAGED_SECTION_RULES: &[SectionRule<'static>] = &[];

fn section_headers_size(header_info: &layout::HeaderInfo) -> u64 {
    object::pe::IMAGE_SIZEOF_SECTION_HEADER as u64
        * u64::from(header_info.num_output_sections_with_content)
}

impl SinglePartSectionId {
    const fn part_id(self) -> PartId {
        PartId::from_u32(self as u32)
    }

    const fn output_section_id(self) -> OutputSectionId {
        OutputSectionId::from_u32(self as u32)
    }
}

impl RegularSectionId {
    const fn output_section_id(self) -> OutputSectionId {
        OutputSectionId::from_u32(COFF_NUM_SINGLE_PART_SECTIONS).offset(self as usize)
    }
}

// The "This program cannot be run in MS-DOS mode." message. There is no good way to combine raw
// bytes (assembly) with readable text without involving additional dependencies.
pub(crate) const DOS_STUB: &[u8] = &[
    0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd, 0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e, 0x6f,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53, 0x20,
    0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x24, 0x00, 0x00,
];
