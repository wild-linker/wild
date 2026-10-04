use crate::bail;
use crate::elf::Elf64;
use crate::ensure;
use crate::error::Context as _;
use crate::error::Result;
use crate::platform::Platform;
use crate::platform::PreviousRelocationInfo;
use linker_utils::bit_misc::BitExtraction;
use linker_utils::elf::DynamicRelocationKind;
use linker_utils::elf::RelocationKindInfo;
use linker_utils::elf::ppc64_rel_type_to_string;
use linker_utils::ppc64::RelaxationKind;
use linker_utils::relaxation::RelocationModifier;

pub(crate) struct ElfPpc64;

impl crate::platform::Arch for ElfPpc64 {
    type Relaxation = Relaxation;
    type Platform = Elf64;

    fn arch_identifier() -> <Self::Platform as Platform>::ArchIdentifier {
        object::elf::EM_PPC64
    }

    #[inline(always)]
    fn relocation_from_raw(r_type: object::elf::RelocationType) -> Result<RelocationKindInfo> {
        linker_utils::ppc64::relocation_type_from_raw(r_type).with_context(|| {
            format!(
                "Unsupported relocation type {}",
                Self::rel_type_to_string(r_type)
            )
        })
    }

    fn is_disallowed_for_interposable_symbols(r_type: object::elf::RelocationType) -> bool {
        matches!(r_type, object::elf::R_PPC64_ADDR32)
    }

    fn get_dynamic_relocation_type(
        relocation: DynamicRelocationKind,
    ) -> object::elf::RelocationType {
        relocation.ppc64_r_type()
    }

    fn rel_type_to_string(r_type: object::elf::RelocationType) -> std::borrow::Cow<'static, str> {
        ppc64_rel_type_to_string(r_type)
    }

    fn write_plt_entry(
        _plt_entry: &mut [u8],
        _got_address: u64,
        _plt_address: u64,
    ) -> crate::error::Result {
        bail!("ppc64 PLT stubs address the GOT from the TOC. The TOC base is required");
    }

    fn write_plt_entry_with_toc(
        plt_entry: &mut [u8],
        got_address: u64,
        _plt_address: u64,
        toc_base: u64,
    ) -> crate::error::Result {
        let offset = (got_address as i64).wrapping_sub(toc_base as i64);
        ensure!(
            (-0x8000..0x8000).contains(&offset) && (offset & 3) == 0,
            "ppc64 PLT stub GOT displacement {offset:#x} from TOC {toc_base:#x} does not fit in a signed 16-bit DS immediate"
        );
        let disp = (offset as u32) & 0xfffc;
        let insns = [
            0xf841_0018u32,     // std r2, 24(r1)
            0xe982_0000 | disp, // ld r12, disp(r2)
            0x7d89_03a6,        // mtctr r12
            0x4e80_0420,        // bctr
        ];
        let (slots, _) = plt_entry.as_chunks_mut::<4>();
        ensure!(slots.len() >= insns.len(), "ppc64 PLT entry is 16 bytes");
        for (dest, insn) in slots.iter_mut().zip(insns) {
            *dest = insn.to_le_bytes();
        }
        Ok(())
    }

    fn restore_toc_after_plt_call(code: &mut [u8], branch_offset: usize) -> Result {
        if branch_offset
            .checked_add(4)
            .is_none_or(|end| end > code.len())
        {
            bail!("call lacks nop, can't restore toc");
        }
        let insn = u32::from_le_bytes(code[branch_offset..branch_offset + 4].try_into().unwrap());
        // LK is the low bit. A tail `b` does not return, so it has no restore slot.
        if insn & 1 != 1 {
            return Ok(());
        }
        let Some(next) = branch_offset.checked_add(4) else {
            bail!("call lacks nop, can't restore toc");
        };
        if next + 4 > code.len() {
            bail!("call lacks nop, can't restore toc");
        }
        let following = u32::from_le_bytes(code[next..next + 4].try_into().unwrap());
        if following != 0x6000_0000 {
            bail!("call lacks nop, can't restore toc");
        }
        code[next..next + 4].copy_from_slice(&0xe841_0018u32.to_le_bytes());
        Ok(())
    }

    fn absolute_ifunc_needs_irelative(
        output_kind: crate::output_kind::OutputKind,
        _section_is_writable: bool,
    ) -> bool {
        output_kind.needs_dynamic()
    }

    /// The thread pointer (`r13`) points 0x7000 bytes past the start of the static TLS block.
    fn tp_offset_start(layout: &crate::layout::Layout<Elf64>) -> u64 {
        layout.tls_start_address() + 0x7000
    }

    /// DTV entries point 0x8000 bytes past the start of each module's TLS block.
    fn get_dtv_offset() -> u64 {
        0x8000
    }

    fn get_property_class(_property_type: u32) -> Option<crate::elf::PropertyClass> {
        None
    }

    fn merge_eflags(
        eflags: impl Iterator<Item = object::elf::FileFlags>,
    ) -> Result<object::elf::FileFlags> {
        Ok(eflags.fold(object::elf::FileFlags(0), |merged, flags| merged | flags))
    }

    fn high_part_relocations() -> &'static [object::elf::RelocationType] {
        &[]
    }

    fn local_entry_offset(st_other: u8) -> u64 {
        // ELFv2 encodes the distance from a function's global entry point to its local entry point
        // in st_other bits [7:5]: offset = ((1 << k) >> 2) << 2 (k=0,1 -> 0; 2 -> 4; 3 -> 8; ...).
        let bit = u32::from(object::elf::STO_PPC64_LOCAL_BIT);
        let k = u64::from(st_other).extract_bit_range(bit..bit + 3);
        u64::from(((1u32 << k) >> 2) << 2)
    }

    #[allow(unused_variables)]
    #[inline(always)]
    fn new_relaxation(
        relocation_kind: object::elf::RelocationType,
        section_bytes: &[u8],
        offset_in_section: u64,
        flags: crate::value_flags::ValueFlags,
        output_kind: crate::output_kind::OutputKind,
        section_flags: linker_utils::elf::SectionFlags,
        relax_deltas: Option<&linker_utils::relaxation::SectionRelaxDeltas>,
        _sym_addr: u64,
        _section_address: u64,
        _rel_addend: i64,
        _previous_relocation: Option<PreviousRelocationInfo<object::elf::RelocationType>>,
    ) -> Option<Self::Relaxation>
    where
        Self: std::marker::Sized,
    {
        None
    }

    fn get_source_info<'data>(
        object: &<Self::Platform as Platform>::File<'data>,
        relocations: &<Self::Platform as Platform>::RelocationSections,
        section: &<Self::Platform as Platform>::SectionHeader,
        offset_in_section: u64,
    ) -> Result<crate::platform::SourceInfo> {
        crate::dwarf_address_info::get_source_info::<crate::elf::Class64, Self>(
            object,
            relocations,
            section,
            offset_in_section,
        )
    }
}

// Relaxations are not yet implemented for ppc64, so `new_relaxation` always returns `None` and
// this type is never constructed.
#[derive(Debug, Clone)]
pub(crate) struct Relaxation {
    kind: RelaxationKind,
    rel_info: RelocationKindInfo,
    mandatory: bool,
}

impl crate::platform::Relaxation for Relaxation {
    fn apply(&self, section_bytes: &mut [u8], offset_in_section: &mut u64, addend: &mut i64) {
        self.kind.apply(section_bytes, offset_in_section, addend);
    }

    fn rel_info(&self) -> RelocationKindInfo {
        self.rel_info
    }

    fn debug_kind(&self) -> impl std::fmt::Debug {
        &self.kind
    }

    fn next_modifier(&self) -> RelocationModifier {
        self.kind.next_modifier()
    }

    fn is_mandatory(&self) -> bool {
        self.mandatory
    }
}
