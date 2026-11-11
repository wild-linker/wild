use crate::coff;
use crate::coff::Coff;
use crate::platform::PreviousRelocationInfo;
use linker_utils::elf::RelocationKindInfo;
use linker_utils::relaxation::RelocationModifier;

pub(crate) struct CoffX86_64;

impl crate::platform::Arch for CoffX86_64 {
    type Relaxation = Relaxation;

    type Platform = Coff;

    fn arch_identifier() -> <Self::Platform as crate::platform::Platform>::ArchIdentifier {}

    fn get_dynamic_relocation_type(
        _relocation: linker_utils::elf::DynamicRelocationKind,
    ) -> coff::RelocationType {
        unreachable!()
    }

    fn write_plt_entry(
        _plt_entry: &mut [u8],
        _got_address: u64,
        _plt_address: u64,
    ) -> crate::error::Result {
        unreachable!()
    }

    fn relocation_from_raw(
        _r_type: <Self::Platform as crate::platform::Platform>::RelocationInfo,
    ) -> crate::error::Result<RelocationKindInfo> {
        todo!()
    }

    fn rel_type_to_string(_r_type: coff::RelocationType) -> std::borrow::Cow<'static, str> {
        todo!()
    }

    fn tp_offset_start(_layout: &crate::layout::Layout<Self::Platform>) -> u64 {
        todo!()
    }

    fn get_property_class(_property_type: u32) -> Option<crate::elf::PropertyClass> {
        unreachable!()
    }

    fn merge_eflags(_eflags: impl Iterator<Item = u32>) -> crate::error::Result<u32> {
        unreachable!()
    }

    fn high_part_relocations() -> &'static [coff::RelocationType] {
        unreachable!()
    }

    fn get_source_info<'data>(
        _object: &<Self::Platform as crate::platform::Platform>::File<'data>,
        _relocations: &<Self::Platform as crate::platform::Platform>::RelocationSections,
        _section: &<Self::Platform as crate::platform::Platform>::SectionHeader,
        _offset_in_section: u64,
    ) -> crate::error::Result<crate::platform::SourceInfo> {
        todo!()
    }

    fn new_relaxation(
        _relocation_kind: coff::RelocationType,
        _section_bytes: &[u8],
        _offset_in_section: u64,
        _flags: crate::value_flags::ValueFlags,
        _output_kind: crate::output_kind::OutputKind,
        _section_flags: <Self::Platform as crate::platform::Platform>::SectionFlags,
        _relax_deltas: Option<&linker_utils::relaxation::SectionRelaxDeltas>,
        _sym_addr: u64,
        _section_address: u64,
        _rel_addend: i64,
        _previous_relocation: Option<PreviousRelocationInfo<coff::RelocationType>>,
    ) -> Option<Self::Relaxation> {
        None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Relaxation {}

impl crate::platform::Relaxation for Relaxation {
    fn apply(&self, _section_bytes: &mut [u8], _offset_in_section: &mut u64, _addend: &mut i64) {}

    fn rel_info(&self) -> RelocationKindInfo {
        todo!()
    }

    fn debug_kind(&self) -> impl std::fmt::Debug {}

    fn next_modifier(&self) -> RelocationModifier {
        unreachable!()
    }

    fn is_mandatory(&self) -> bool {
        todo!()
    }

    fn needs_got_fallback(&self) -> bool {
        todo!()
    }
}
