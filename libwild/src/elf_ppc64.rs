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
use linker_utils::elf::shf;
use linker_utils::ppc64::RelaxationKind;
use linker_utils::ppc64::relocation_type_from_raw;
use linker_utils::ppc64::tls_xform_to_dform;
use linker_utils::relaxation::RelocationModifier;

macro_rules! rel_info_from_type {
    ($r_type:expr) => {
        const { relocation_type_from_raw($r_type).unwrap() }
    };
}

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
        matches!(r_type, object::elf::R_PPC64_ADDR32) || is_tprel(r_type)
    }

    fn is_disallowed_in_shared_object(r_type: object::elf::RelocationType) -> bool {
        is_tprel(r_type)
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
        let Some(ha) = offset
            .checked_add(0x8000)
            .and_then(|adjusted| i16::try_from(adjusted >> 16).ok())
        else {
            bail!(
                "ppc64 PLT stub GOT displacement {offset:#x} from TOC {toc_base:#x} does not fit in a signed 32-bit TOC-relative immediate"
            );
        };
        ensure!(
            (offset & 3) == 0,
            "ppc64 PLT stub GOT displacement {offset:#x} from TOC {toc_base:#x} is not 4-byte aligned"
        );
        let ha = u32::from(ha as u16);
        let lo = (offset as u32) & 0xfffc;
        let insns = [
            0xf841_0018u32,   // std r2, 24(r1)
            0x3d82_0000 | ha, // addis r12, r2, offset@ha
            0xe98c_0000 | lo, // ld r12, offset@l(r12)
            0x7d89_03a6,      // mtctr r12
            0x4e80_0420,      // bctr
            0x6000_0000,      // nop
            0x6000_0000,      // nop
            0x6000_0000,      // nop
        ];
        let (slots, _) = plt_entry.as_chunks_mut::<4>();
        ensure!(slots.len() >= insns.len(), "ppc64 PLT entry is 32 bytes");
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

    /// DTPREL is relative to that DTV point for both executables and shared objects.
    fn dtp_offset_base(layout: &crate::layout::Layout<Elf64>, _for_shared_object: bool) -> u64 {
        layout
            .tls_start_address()
            .wrapping_add(Self::get_dtv_offset())
    }

    fn executable_tlsld_offset(_layout: &crate::layout::Layout<Elf64>) -> u64 {
        // `__tls_get_addr` adds the 0x8000 DTV bias. A zero slot makes it return the DTP, and the
        // compiler then adds the symbol's DTPREL.
        0
    }

    fn get_property_class(_property_type: u32) -> Option<crate::elf::PropertyClass> {
        None
    }

    fn merge_eflags(
        eflags: impl Iterator<Item = object::elf::FileFlags>,
    ) -> Result<object::elf::FileFlags> {
        const EF_PPC64_ABI_V2: u32 = 2;

        for flags in eflags {
            if (flags.0 & !object::elf::EF_PPC64_ABI) != 0 {
                bail!("Unrecognized ppc64 e_flags: {:#x}", flags.0);
            }
            match flags.ppc64_abi() {
                0 | EF_PPC64_ABI_V2 => {}
                1 => bail!("PPC64 ABI version 1 is not supported"),
                _ => bail!("Unrecognized ppc64 e_flags: {:#x}", flags.0),
            }
        }
        Ok(object::elf::FileFlags(EF_PPC64_ABI_V2))
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

    #[inline(always)]
    fn new_relaxation(
        relocation_kind: object::elf::RelocationType,
        section_bytes: &[u8],
        offset_in_section: u64,
        flags: crate::value_flags::ValueFlags,
        output_kind: crate::output_kind::OutputKind,
        section_flags: linker_utils::elf::SectionFlags,
        _relax_deltas: Option<&linker_utils::relaxation::SectionRelaxDeltas>,
        _sym_addr: u64,
        _section_address: u64,
        _rel_addend: i64,
        _previous_relocation: Option<PreviousRelocationInfo<object::elf::RelocationType>>,
    ) -> Option<Self::Relaxation>
    where
        Self: std::marker::Sized,
    {
        if !section_flags.contains(shf::EXECINSTR) || !output_kind.is_executable() {
            return None;
        }

        let offset = usize::try_from(offset_in_section).ok()?;
        let insn = u32::from_le_bytes(section_bytes.get(offset..offset + 4)?.try_into().ok()?);
        let interposable = flags.is_interposable();
        let mandatory_for_static = output_kind.is_static_executable();

        let (kind, rel_info, mandatory) = match relocation_kind {
            // General Dynamic -> Local Exec (local TLS symbol in executable).
            object::elf::R_PPC64_GOT_TLSGD16_HA | object::elf::R_PPC64_GOT_TLSGD16_HI
                if !interposable && is_addis(insn) =>
            {
                (
                    RelaxationKind::ReplaceWithNop,
                    rel_info_from_type!(object::elf::R_PPC64_NONE),
                    mandatory_for_static,
                )
            }
            object::elf::R_PPC64_GOT_TLSGD16_LO | object::elf::R_PPC64_GOT_TLSGD16
                if !interposable && is_addi_r3(insn) =>
            {
                (
                    RelaxationKind::AddisR3R13,
                    rel_info_from_type!(object::elf::R_PPC64_TPREL16_HA),
                    mandatory_for_static,
                )
            }
            object::elf::R_PPC64_TLSGD if !interposable && is_tls_get_addr_call(insn) => (
                RelaxationKind::AddiR3R3SkipNext,
                rel_info_from_type!(object::elf::R_PPC64_TPREL16_LO),
                mandatory_for_static,
            ),
            object::elf::R_PPC64_TLSGD if !interposable && is_tls_get_addr_plt_setup(insn) => (
                RelaxationKind::ReplaceWithNopSkipNext,
                rel_info_from_type!(object::elf::R_PPC64_NONE),
                mandatory_for_static,
            ),

            // General Dynamic -> Initial Exec (interposable TLS symbol in dynamic executable).
            object::elf::R_PPC64_GOT_TLSGD16_HA if interposable && is_addis(insn) => (
                RelaxationKind::NoOp,
                rel_info_from_type!(object::elf::R_PPC64_GOT_TPREL16_HA),
                false,
            ),
            object::elf::R_PPC64_GOT_TLSGD16_HI if interposable && is_addis(insn) => (
                RelaxationKind::NoOp,
                rel_info_from_type!(object::elf::R_PPC64_GOT_TPREL16_HI),
                false,
            ),
            object::elf::R_PPC64_GOT_TLSGD16_LO if interposable && is_addi_r3(insn) => (
                RelaxationKind::AddiToLd,
                rel_info_from_type!(object::elf::R_PPC64_GOT_TPREL16_LO_DS),
                false,
            ),
            object::elf::R_PPC64_GOT_TLSGD16 if interposable && is_addi_r3(insn) => (
                RelaxationKind::AddiToLd,
                rel_info_from_type!(object::elf::R_PPC64_GOT_TPREL16_DS),
                false,
            ),
            object::elf::R_PPC64_TLSGD if interposable && is_tls_get_addr_call(insn) => (
                RelaxationKind::AddR3R3R13SkipNext,
                rel_info_from_type!(object::elf::R_PPC64_NONE),
                false,
            ),
            object::elf::R_PPC64_TLSGD if interposable && is_tls_get_addr_plt_setup(insn) => (
                RelaxationKind::ReplaceWithNopSkipNext,
                rel_info_from_type!(object::elf::R_PPC64_NONE),
                false,
            ),

            // Local Dynamic -> Local Exec (always targets the executable's own TLS module).
            object::elf::R_PPC64_GOT_TLSLD16_HA | object::elf::R_PPC64_GOT_TLSLD16_HI
                if is_addis(insn) =>
            {
                (
                    RelaxationKind::ReplaceWithNop,
                    rel_info_from_type!(object::elf::R_PPC64_NONE),
                    mandatory_for_static,
                )
            }
            object::elf::R_PPC64_GOT_TLSLD16_LO | object::elf::R_PPC64_GOT_TLSLD16
                if is_addi(insn) =>
            {
                (
                    RelaxationKind::ReplaceWithNop,
                    rel_info_from_type!(object::elf::R_PPC64_NONE),
                    mandatory_for_static,
                )
            }
            object::elf::R_PPC64_TLSLD if is_tls_get_addr_call(insn) => (
                RelaxationKind::AddiR3R13DtpBiasSkipNext,
                rel_info_from_type!(object::elf::R_PPC64_NONE),
                mandatory_for_static,
            ),
            object::elf::R_PPC64_TLSLD if is_tls_get_addr_plt_setup(insn) => (
                RelaxationKind::ReplaceWithNopSkipNext,
                rel_info_from_type!(object::elf::R_PPC64_NONE),
                mandatory_for_static,
            ),

            // Initial Exec -> Local Exec (local TLS symbol in executable).
            object::elf::R_PPC64_GOT_TPREL16_HA | object::elf::R_PPC64_GOT_TPREL16_HI
                if !interposable && is_addis(insn) =>
            {
                (
                    RelaxationKind::ReplaceWithNop,
                    rel_info_from_type!(object::elf::R_PPC64_NONE),
                    false,
                )
            }
            object::elf::R_PPC64_GOT_TPREL16_LO_DS | object::elf::R_PPC64_GOT_TPREL16_DS
                if !interposable && is_ld(insn) =>
            {
                (
                    RelaxationKind::LdToAddisR13,
                    rel_info_from_type!(object::elf::R_PPC64_TPREL16_HA),
                    false,
                )
            }
            object::elf::R_PPC64_TLS if !interposable => {
                let (_, is_ds) = tls_xform_to_dform(insn)?;
                let rel_info = if is_ds {
                    rel_info_from_type!(object::elf::R_PPC64_TPREL16_LO_DS)
                } else {
                    rel_info_from_type!(object::elf::R_PPC64_TPREL16_LO)
                };
                (RelaxationKind::TlsXFormToDForm, rel_info, false)
            }

            _ => return None,
        };

        Some(Relaxation {
            kind,
            rel_info,
            mandatory,
        })
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

fn is_addis(insn: u32) -> bool {
    (insn >> 26) == 15
}

fn is_addi(insn: u32) -> bool {
    (insn >> 26) == 14
}

fn is_addi_r3(insn: u32) -> bool {
    // `addi r3, rA, imm`: opcode 14 (0x38), RT = 3 (0x0060_0000).
    (insn & 0xffe0_0000) == 0x3860_0000
}

fn is_ld(insn: u32) -> bool {
    // `ld rD, ds(rA)`: opcode 58 (0xe8), XO = 0.
    (insn & 0xfc00_0003) == 0xe800_0000
}

fn is_tls_get_addr_call(insn: u32) -> bool {
    // Direct `bl`/`b` (opcode 18, AA=0) or `-fno-plt` indirect `bctrl`/`bctr`.
    (insn & 0xfc00_0002) == 0x4800_0000 || insn == 0x4e80_0421 || insn == 0x4e80_0420
}

fn is_tls_get_addr_plt_setup(insn: u32) -> bool {
    // `-fno-plt` setup instructions loading `__tls_get_addr`: `addis`, `addi`, `ld`, `mtctr`.
    is_addis(insn) || is_addi(insn) || is_ld(insn) || (insn & 0xfc1f_ffff) == 0x7c09_03a6
}

fn is_tprel(r_type: object::elf::RelocationType) -> bool {
    matches!(
        r_type,
        object::elf::R_PPC64_TPREL16
            | object::elf::R_PPC64_TPREL16_LO
            | object::elf::R_PPC64_TPREL16_HI
            | object::elf::R_PPC64_TPREL16_HA
            | object::elf::R_PPC64_TPREL16_DS
            | object::elf::R_PPC64_TPREL16_LO_DS
            | object::elf::R_PPC64_TPREL64
    )
}

#[derive(Debug, Clone)]
pub(crate) struct Relaxation {
    kind: RelaxationKind,
    rel_info: RelocationKindInfo,
    mandatory: bool,
}

impl crate::platform::Relaxation for Relaxation {
    type RelocationKindInfo = RelocationKindInfo;

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
