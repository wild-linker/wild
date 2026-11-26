use crate::ArchKind;
use crate::Result;
use crate::arch::Arch;
use crate::arch::Instruction;
use crate::arch::Relaxation;
use crate::arch::RelaxationByteRange;
use crate::asm_diff::BasicValueKind;
use crate::utils::decode_insn_with_objdump;
use linker_utils::elf::DynamicRelocationKind;
use linker_utils::elf::RelocationKindInfo;
use linker_utils::elf::ppc64_rel_type_to_string;
use linker_utils::ppc64::RelaxationKind;
use linker_utils::relaxation::RelocationModifier;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Ppc64;

impl Arch for Ppc64 {
    type RType = RType;
    type RelaxationKind = RelaxationKind;
    type RawInstruction = Option<String>;

    const MAX_RELAX_MODIFY_BEFORE: u64 = 0;
    const MAX_RELAX_MODIFY_AFTER: u64 = 4;

    fn possible_relaxations_do(
        r_type: Self::RType,
        _section_kind: object::SectionKind,
        mut cb: impl FnMut(Relaxation<Self>),
    ) {
        let mut relax = |relaxation_kind, new_r_type| {
            cb(Relaxation {
                relaxation_kind,
                new_r_type: RType(new_r_type),
                alt_r_type: None,
            });
        };

        match r_type.0 {
            object::elf::R_PPC64_GOT_TLSGD16_HA => {
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
                relax(RelaxationKind::NoOp, object::elf::R_PPC64_GOT_TPREL16_HA);
            }
            object::elf::R_PPC64_GOT_TLSGD16_HI => {
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
                relax(RelaxationKind::NoOp, object::elf::R_PPC64_GOT_TPREL16_HI);
            }
            object::elf::R_PPC64_GOT_TLSGD16_LO => {
                relax(RelaxationKind::AddisR3R13, object::elf::R_PPC64_TPREL16_HA);
                relax(
                    RelaxationKind::AddiToLd,
                    object::elf::R_PPC64_GOT_TPREL16_LO_DS,
                );
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
            }
            object::elf::R_PPC64_GOT_TLSGD16 => {
                relax(RelaxationKind::AddisR3R13, object::elf::R_PPC64_TPREL16_HA);
                relax(
                    RelaxationKind::AddiToLd,
                    object::elf::R_PPC64_GOT_TPREL16_DS,
                );
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
            }
            object::elf::R_PPC64_TLSGD => {
                relax(
                    RelaxationKind::AddiR3R3SkipNext,
                    object::elf::R_PPC64_TPREL16_LO,
                );
                relax(
                    RelaxationKind::AddR3R3R13SkipNext,
                    object::elf::R_PPC64_NONE,
                );
                relax(
                    RelaxationKind::ReplaceWithNopSkipNext,
                    object::elf::R_PPC64_NONE,
                );
            }
            object::elf::R_PPC64_GOT_TLSLD16_HA
            | object::elf::R_PPC64_GOT_TLSLD16_HI
            | object::elf::R_PPC64_GOT_TLSLD16_LO
            | object::elf::R_PPC64_GOT_TLSLD16 => {
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
            }
            object::elf::R_PPC64_TLSLD => {
                relax(
                    RelaxationKind::AddiR3R13DtpBiasSkipNext,
                    object::elf::R_PPC64_NONE,
                );
                relax(
                    RelaxationKind::ReplaceWithNopSkipNext,
                    object::elf::R_PPC64_NONE,
                );
            }
            object::elf::R_PPC64_GOT_TPREL16_HA | object::elf::R_PPC64_GOT_TPREL16_HI => {
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
            }
            object::elf::R_PPC64_GOT_TPREL16_LO_DS | object::elf::R_PPC64_GOT_TPREL16_DS => {
                relax(
                    RelaxationKind::LdToAddisR13,
                    object::elf::R_PPC64_TPREL16_HA,
                );
                relax(RelaxationKind::ReplaceWithNop, object::elf::R_PPC64_NONE);
            }
            object::elf::R_PPC64_TLS => {
                relax(
                    RelaxationKind::TlsXFormToDForm,
                    object::elf::R_PPC64_TPREL16_LO,
                );
                relax(
                    RelaxationKind::TlsXFormToDForm,
                    object::elf::R_PPC64_TPREL16_LO_DS,
                );
            }
            _ => {}
        }
    }

    fn relaxation_byte_range(_relaxation: Relaxation<Self>) -> RelaxationByteRange {
        RelaxationByteRange {
            offset_shift: 0,
            num_bytes: 4,
        }
    }

    fn apply_relaxation(
        relaxation_kind: Self::RelaxationKind,
        section_bytes: &mut [u8],
        offset_in_section: &mut u64,
        addend: &mut i64,
    ) -> Result<()> {
        relaxation_kind.apply(section_bytes, offset_in_section, addend);
        Ok(())
    }

    fn next_relocation_modifier(relaxation_kind: Self::RelaxationKind) -> RelocationModifier {
        relaxation_kind.next_modifier()
    }

    fn instruction_to_string(instruction: &Instruction<Self>) -> String {
        instruction.raw_instruction.clone().unwrap_or_default()
    }

    fn decode_instructions_in_range(
        section_bytes: &[u8],
        section_address: u64,
        _function_offset_in_section: u64,
        range: std::ops::Range<u64>,
    ) -> Vec<Instruction<'_, Self>> {
        // ppc64 instructions are a fixed 4 bytes, so we can start decoding at the (aligned) start
        // of the range.
        let mut offset = range.start & !3;
        let mut instructions = Vec::new();
        while offset < range.end {
            if offset as usize + 4 > section_bytes.len() {
                break;
            }
            let bytes = &section_bytes[offset as usize..offset as usize + 4];
            let address = section_address + offset;
            let raw_instruction = decode_insn_with_objdump(bytes, address, ArchKind::Ppc64).ok();
            instructions.push(Instruction {
                raw_instruction,
                address,
                bytes,
            });
            offset += 4;
        }
        instructions
    }

    fn decode_plt_entry(
        _plt_entry: &[u8],
        _plt_base: u64,
        _plt_offset: u64,
    ) -> Option<crate::arch::PltEntry> {
        // The stub addresses the GOT from the TOC (`r2`) with `addis`/`ld`. This decoder does not
        // have the TOC base, so the entry cannot be turned back into an address.
        None
    }

    fn should_chain_relocations(_chain_prefix: &[Self::RType]) -> bool {
        false
    }

    fn get_relocation_base_mask(_relocation_info: &RelocationKindInfo) -> u64 {
        u64::MAX
    }

    fn relocation_to_pc_offset(_relocation_info: &RelocationKindInfo) -> u64 {
        0
    }

    fn is_complete_chain(_chain: impl Iterator<Item = Self::RType>) -> bool {
        true
    }

    fn get_basic_value_for_tp_offset() -> BasicValueKind {
        BasicValueKind::TlsOffset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RType(object::elf::RelocationType);

impl crate::arch::RType for RType {
    fn from_raw(raw: object::elf::RelocationType) -> Self {
        RType(raw)
    }

    fn from_dynamic_relocation_kind(kind: DynamicRelocationKind) -> Self {
        Self::from_raw(kind.ppc64_r_type())
    }

    fn opt_relocation_info(self) -> Option<RelocationKindInfo> {
        linker_utils::ppc64::relocation_type_from_raw(self.0)
    }

    fn dynamic_relocation_kind(self) -> Option<DynamicRelocationKind> {
        DynamicRelocationKind::from_ppc64_r_type(self.0)
    }
}

impl Display for RType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&ppc64_rel_type_to_string(self.0), f)
    }
}

impl crate::arch::RelaxationKind for RelaxationKind {
    fn is_no_op(self) -> bool {
        matches!(self, RelaxationKind::NoOp)
    }

    fn is_replace_with_no_op(self) -> bool {
        matches!(
            self,
            RelaxationKind::ReplaceWithNop | RelaxationKind::ReplaceWithNopSkipNext
        )
    }
}
