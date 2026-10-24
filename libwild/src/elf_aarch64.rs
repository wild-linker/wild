use crate::alignment::Alignment;
use crate::elf::Elf64;
use crate::elf::PLT_ENTRY_SIZE;
use crate::elf::PropertyClass;
use crate::elf::output_section_id;
use crate::ensure;
use crate::error::Context as _;
use crate::error::Result;
use crate::layout::FileLayoutState;
use crate::layout::GroupState;
use crate::layout::HandlerData as _;
use crate::layout::Layout;
use crate::malfunction_point_ret;
use crate::platform::ObjectFile as _;
use crate::platform::Platform;
use crate::platform::PreviousRelocationInfo;
use crate::platform::SectionAttributes as _;
use crate::resolution::SectionSlot;
use crate::symbol_db::SymbolId;
use crate::value_flags::FlagsForSymbol as _;
use crate::value_flags::ValueFlags;
use linker_utils::aarch64::RelaxationKind;
use linker_utils::aarch64::relocation_type_from_raw;
use linker_utils::bit_misc::BitExtraction;
use linker_utils::elf::AArch64Instruction;
use linker_utils::elf::AllowedRange;
use linker_utils::elf::DynamicRelocationKind;
use linker_utils::elf::PAGE_MASK_4KB;
use linker_utils::elf::RelocationKind;
use linker_utils::elf::RelocationKindInfo;
use linker_utils::elf::SIZE_4KB;
use linker_utils::elf::Sign;
use linker_utils::elf::aarch64_rel_type_to_string;
use linker_utils::elf::shf;
use linker_utils::relaxation::RelocationModifier;
use object::elf::GNU_PROPERTY_AARCH64_FEATURE_1_AND;

pub(crate) struct ElfAArch64;

const PLT_ENTRY_TEMPLATE: &[u8] = &[
    0x10, 0x00, 0x00, 0x90, // adrp x16, page(&(.got.plt[n]))
    0x11, 0x02, 0x40, 0xf9, // ldr x17, [x16, offset(&(.got.plt[n]))]
    0x20, 0x02, 0x1f, 0xd6, // br x17
    0x1f, 0x20, 0x03, 0xd5, // nop
];

// ADRP+ADD+BR thunk template.
const THUNK_TEMPLATE: &[u8] = &[
    0x10, 0x00, 0x00, 0x90, // ADRP x16, 0
    0x10, 0x02, 0x00, 0x91, // ADD  x16, x16, #0
    0x00, 0x02, 0x1F, 0xD6, // BR   x16
];

/// The shortest range-limited branch for this arch.
pub(crate) const MIN_BRANCH_RANGE: u64 = 128 * 1024 * 1024;

/// `bti c`: a valid landing pad for indirect calls and for branches through x16/x17.
pub(crate) const BTI_C: u32 = 0xd503_245f;
/// `bti jc`: a valid landing pad for both indirect calls and indirect jumps.
pub(crate) const BTI_JC: u32 = 0xd503_24df;
/// `paciasp`: implicitly a landing pad for indirect calls and branches through x16/x17.
pub(crate) const PACIASP: u32 = 0xd503_233f;
/// `pacibsp`: implicitly a landing pad for indirect calls and branches through x16/x17.
pub(crate) const PACIBSP: u32 = 0xd503_237f;

/// Whether the instruction at the start of `bytes` is a valid target for an indirect branch through
/// x16/x17 when BTI is enabled.
fn is_bti_landing_pad(bytes: &[u8]) -> bool {
    bytes.first_chunk::<4>().is_some_and(|insn| {
        matches!(
            u32::from_le_bytes(*insn),
            BTI_C | BTI_JC | PACIASP | PACIBSP
        )
    })
}

/// Returns the thunk targets that need a BTI landing pad, grouped by the input section that each
/// pad is placed immediately before. Targets that already start with a landing pad are skipped.
fn collect_bti_pad_targets(
    thunk_blocks: &[crate::thunks::ThunkBlock],
    group_states: &[GroupState<Elf64>],
    symbol_db: &crate::symbol_db::SymbolDb<Elf64>,
    per_symbol_flags: &crate::value_flags::PerSymbolFlags,
    output_sections: &crate::output_section_id::OutputSections<Elf64>,
) -> Result<crate::thunks::LandingPadPlan> {
    let mut plan = crate::thunks::LandingPadPlan::default();

    let targets: std::collections::BTreeSet<SymbolId> = thunk_blocks
        .iter()
        .flat_map(|block| block.symbols.iter().copied())
        .collect();

    for symbol_id in targets {
        if per_symbol_flags
            .flags_for_symbol(symbol_id)
            .intersects(ValueFlags::IFUNC | ValueFlags::DYNAMIC | ValueFlags::PLT)
        {
            continue;
        }

        let file_id = symbol_db.file_id_for_symbol(symbol_id);
        let Some(FileLayoutState::Object(obj)) =
            group_states[file_id.group()].files.get(file_id.file())
        else {
            continue;
        };

        let symbol_index = obj.symbol_id_range().id_to_input(symbol_id);
        let symbol = obj.object.symbol(symbol_index)?;

        let Some(section_index) = obj.object.symbol_section(symbol, symbol_index)? else {
            continue;
        };

        // Pads are placed while laying out each object's sections. Linker-script-sorted sections
        // are laid out in a separate pass, so pads aren't supported for targets in them.
        let in_sorted_section = match obj.sections.get(section_index.0) {
            Some(SectionSlot::Loaded(_)) => false,
            Some(SectionSlot::Sorted(_)) => true,
            _ => continue,
        };

        let part_id = symbol_db.part_id_for_symbol(symbol_id);
        if !output_sections
            .output_info(part_id.output_section_id::<Elf64>())
            .section_attributes
            .is_executable()
        {
            continue;
        }

        let section = obj.object.section(section_index)?;
        let symbol_offset = obj.object.symbol_offset_in_section(symbol, section_index)?;

        // Targets that already begin with a valid landing pad don't need one.
        let section_data = obj.object.raw_section_data(section)?;
        if section_data
            .get(symbol_offset as usize..)
            .is_some_and(is_bti_landing_pad)
        {
            continue;
        }

        ensure!(
            !in_sorted_section,
            "Thunk target `{}` needs a landing pad, but is in a linker-script-sorted section, \
             which is not supported",
            symbol_db.symbol_name_for_display(symbol_id)
        );

        plan.add(
            file_id,
            section_index.0,
            crate::thunks::LandingPad {
                target: symbol_id,
                part_id,
                size: bti_pad_size(part_id, output_sections),
            },
        );
    }

    tracing::debug!(count = plan.len(), "BTI landing pads");

    Ok(plan)
}

/// Reserves space for each BTI landing pad in the group that owns its target.
fn reserve_bti_pad_space(
    plan: &crate::thunks::LandingPadPlan,
    group_states: &mut [GroupState<Elf64>],
    total_sizes: &mut crate::output_section_part_map::OutputSectionPartMap<u64>,
) {
    for (group_index, pad) in plan.iter() {
        group_states[group_index]
            .common
            .allocate(pad.part_id, pad.size);
        total_sizes.increment(pad.part_id, pad.size);
    }
}

/// Bytes reserved for a BTI landing pad (`bti c; b target`), rounded up to the part's alignment.
fn bti_pad_size(
    part_id: crate::part_id::PartId,
    output_sections: &crate::output_section_id::OutputSections<Elf64>,
) -> u64 {
    part_id.alignment(output_sections).align_up(8)
}

const _ASSERTS: () = {
    assert!(PLT_ENTRY_TEMPLATE.len() as u64 == PLT_ENTRY_SIZE);
};

macro_rules! rel_info_from_type {
    ($r_type:expr) => {
        const { relocation_type_from_raw($r_type).unwrap() }
    };
}

impl crate::platform::Arch for ElfAArch64 {
    type Relaxation = Relaxation;
    type Platform = Elf64;

    fn arch_identifier() -> <Self::Platform as Platform>::ArchIdentifier {
        object::elf::EM_AARCH64
    }

    // The table of the relocations is documented here:
    // https://github.com/ARM-software/abi-aa/blob/main/aaelf64/aaelf64.rst.
    #[inline(always)]
    fn relocation_from_raw(r_type: object::elf::RelocationType) -> Result<RelocationKindInfo> {
        linker_utils::aarch64::relocation_type_from_raw(r_type).with_context(|| {
            format!(
                "Unsupported relocation type {}",
                Self::rel_type_to_string(r_type)
            )
        })
    }

    fn undefined_weak_target(r_type: object::elf::RelocationType, place: u64) -> Option<u64> {
        match r_type {
            object::elf::R_AARCH64_CALL26
            | object::elf::R_AARCH64_CONDBR19
            | object::elf::R_AARCH64_JUMP26
            | object::elf::R_AARCH64_TSTBR14 => Some(place.wrapping_add(4)),
            object::elf::R_AARCH64_PREL16
            | object::elf::R_AARCH64_PREL32
            | object::elf::R_AARCH64_PREL64
            | object::elf::R_AARCH64_ADR_PREL_LO21
            | object::elf::R_AARCH64_ADR_PREL_PG_HI21
            | object::elf::R_AARCH64_ADR_PREL_PG_HI21_NC
            | object::elf::R_AARCH64_MOVW_PREL_G0
            | object::elf::R_AARCH64_MOVW_PREL_G0_NC
            | object::elf::R_AARCH64_MOVW_PREL_G1
            | object::elf::R_AARCH64_MOVW_PREL_G1_NC
            | object::elf::R_AARCH64_MOVW_PREL_G2
            | object::elf::R_AARCH64_MOVW_PREL_G2_NC
            | object::elf::R_AARCH64_MOVW_PREL_G3
            | object::elf::R_AARCH64_LD_PREL_LO19
            | object::elf::R_AARCH64_PLT32 => Some(place),
            _ => None,
        }
    }

    fn is_disallowed_for_interposable_symbols(r_type: object::elf::RelocationType) -> bool {
        matches!(
            r_type,
            object::elf::R_AARCH64_ABS32
                | object::elf::R_AARCH64_ABS16
                | object::elf::R_AARCH64_ADD_ABS_LO12_NC
                | object::elf::R_AARCH64_LDST8_ABS_LO12_NC
                | object::elf::R_AARCH64_LDST32_ABS_LO12_NC
                | object::elf::R_AARCH64_LDST64_ABS_LO12_NC
                | object::elf::R_AARCH64_PREL16
                | object::elf::R_AARCH64_PREL32
                | object::elf::R_AARCH64_PREL64
        )
    }

    fn is_disallowed_in_shared_object(r_type: object::elf::RelocationType) -> bool {
        matches!(
            r_type,
            object::elf::R_AARCH64_ABS32 | object::elf::R_AARCH64_ABS16
        )
    }

    fn get_dynamic_relocation_type(
        relocation: DynamicRelocationKind,
    ) -> object::elf::RelocationType {
        relocation.aarch64_r_type()
    }

    fn rel_type_to_string(r_type: object::elf::RelocationType) -> std::borrow::Cow<'static, str> {
        aarch64_rel_type_to_string(r_type)
    }

    fn write_plt_entry(
        plt_entry: &mut [u8],
        got_address: u64,
        plt_address: u64,
    ) -> crate::error::Result {
        // TODO: For simplicity, we assume now the PLT entry precedes the GOT entry, so we can
        // make the offset calculation in the unsigned type.
        debug_assert!(plt_address < got_address);

        plt_entry.copy_from_slice(PLT_ENTRY_TEMPLATE);
        let plt_page_address = plt_address & !PAGE_MASK_4KB;
        let offset = got_address.wrapping_sub(plt_page_address);
        ensure!(offset < (1 << 32), "PLT is more than 4GiB away from GOT");
        AArch64Instruction::Adr.write_to_value(
            // The immediate value represents a distance in pages.
            offset / SIZE_4KB,
            false,
            &mut plt_entry[0..4],
        );
        AArch64Instruction::LdrRegister.write_to_value(
            // The immediate offset is scaled by 8 as we are loading 8 bytes.
            (offset & PAGE_MASK_4KB) / 8,
            false,
            &mut plt_entry[4..8],
        );
        Ok(())
    }

    fn write_plt_entry_with_bti(
        plt_entry: &mut [u8],
        got_address: u64,
        plt_address: u64,
        _toc_base: u64,
        bti_enabled: bool,
    ) -> crate::error::Result {
        if !bti_enabled {
            return Self::write_plt_entry(plt_entry, got_address, plt_address);
        }

        // The BTI instruction precedes ADRP, so relocations must be
        // calculated relative to the new ADRP instruction address.
        ensure!(plt_address <= u64::MAX - 4, "PLT address overflow");
        Self::write_plt_entry(plt_entry, got_address, plt_address + 4)?;

        // Replace ADRP/LDR/BR/NOP with BTI/ADRP/LDR/BR.
        plt_entry.copy_within(0..12, 4);
        plt_entry[..4].copy_from_slice(&BTI_C.to_le_bytes());

        Ok(())
    }

    fn tp_offset_start(layout: &Layout<Elf64>) -> u64 {
        layout.tls_start_address_aarch64()
    }

    fn process_aarch64_build_attributes<'data>(
        object: &<Self::Platform as Platform>::File<'data>,
        format_specific: &mut <Self::Platform as Platform>::ObjectLayoutStateExt<'data>,
        aarch64_attributes_section_index: object::SectionIndex,
    ) -> crate::error::Result {
        format_specific.aarch64_build_attributes = Some(
            crate::elf::process_aarch64_build_attributes(object, aarch64_attributes_section_index)?,
        );
        Ok(())
    }

    fn get_property_class(property_type: u32) -> Option<PropertyClass> {
        match object::elf::GnuPropertyType(property_type) {
            GNU_PROPERTY_AARCH64_FEATURE_1_AND => Some(PropertyClass::And),
            _ => None,
        }
    }

    fn merge_eflags(
        _eflags: impl Iterator<Item = object::elf::FileFlags>,
    ) -> Result<object::elf::FileFlags> {
        Ok(object::elf::FileFlags(0))
    }

    fn high_part_relocations() -> &'static [object::elf::RelocationType] {
        &[]
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
        sym_addr: u64,
        section_address: u64,
        rel_addend: i64,
        previous_relocation: Option<
            PreviousRelocationInfo<<Self::Platform as Platform>::RelocationInfo>,
        >,
    ) -> Option<Self::Relaxation>
    where
        Self: std::marker::Sized,
    {
        let mut relocation = ElfAArch64::relocation_from_raw(relocation_kind).unwrap();
        let interposable = flags.is_interposable();

        // IFuncs cannot be referenced directly, they always need to go via the GOT.
        if flags.is_ifunc() {
            return match relocation_kind {
                object::elf::R_AARCH64_CALL26 | object::elf::R_AARCH64_JUMP26 => {
                    relocation.kind = RelocationKind::PltRelative;
                    return Some(Relaxation {
                        kind: RelaxationKind::NoOp,
                        rel_info: relocation,
                        mandatory: true,
                    });
                }
                _ => None,
            };
        }

        if relocation_kind == object::elf::R_AARCH64_PLT32 && !interposable {
            relocation.kind = RelocationKind::Relative;
            return Some(Relaxation {
                kind: RelaxationKind::NoOp,
                rel_info: relocation,
                mandatory: output_kind.is_static_executable(),
            });
        }

        // All relaxations below only apply to executable code, so we shouldn't attempt them if a
        // relocation is in a non-executable section.
        if !section_flags.contains(shf::EXECINSTR) {
            return None;
        }

        let offset = offset_in_section as usize;

        match relocation_kind {
            object::elf::R_AARCH64_CONDBR19 | object::elf::R_AARCH64_TSTBR14
                if flags.is_undefined_weak() && interposable =>
            {
                relocation.kind = RelocationKind::PltRelative;
                return Some(Relaxation {
                    kind: RelaxationKind::NoOp,
                    rel_info: relocation,
                    mandatory: true,
                });
            }

            object::elf::R_AARCH64_CALL26 | object::elf::R_AARCH64_JUMP26 if !interposable => {
                relocation.kind = RelocationKind::Relative;
                return Some(Relaxation {
                    kind: RelaxationKind::NoOp,
                    rel_info: relocation,
                    mandatory: output_kind.is_static_executable(),
                });
            }

            // Relax TLSDESC to local exec
            object::elf::R_AARCH64_TLSDESC_ADR_PAGE21
                if output_kind.is_executable() && !interposable =>
            {
                debug_assert!(
                    section_bytes[offset..].starts_with(TLSDESC_ADR_PAGE21_INSN_SEQUENCE),
                    "Unknown R_AARCH64_TLSDESC_ADR_PAGE21 instruction"
                );
                return Some(Relaxation {
                    kind: RelaxationKind::ReplaceWithNop,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_NONE),
                    mandatory: output_kind.is_static_executable(),
                });
            }
            object::elf::R_AARCH64_TLSDESC_LD64_LO12
                if output_kind.is_executable() && !interposable =>
            {
                return Some(Relaxation {
                    kind: RelaxationKind::ReplaceWithNop,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_NONE),
                    mandatory: output_kind.is_static_executable(),
                });
            }
            object::elf::R_AARCH64_TLSDESC_ADD_LO12
                if output_kind.is_executable() && !interposable =>
            {
                debug_assert!(
                    section_bytes[offset..].starts_with(TLSDESC_ADD_LO12_INSN_SEQUENCE),
                    "Unknown R_AARCH64_TLSDESC_ADD_LO12 instruction"
                );
                malfunction_point_ret!("no-movzx0lsl16", None);
                return Some(Relaxation {
                    kind: RelaxationKind::MovzX0Lsl16,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_TLSLE_MOVW_TPREL_G1),
                    mandatory: output_kind.is_static_executable(),
                });
            }
            object::elf::R_AARCH64_TLSDESC_CALL if output_kind.is_executable() && !interposable => {
                return Some(Relaxation {
                    kind: RelaxationKind::MovkX0,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_TLSLE_MOVW_TPREL_G0_NC),
                    mandatory: output_kind.is_static_executable(),
                });
            }

            // Relax TLSDESC to initial exec
            object::elf::R_AARCH64_TLSDESC_ADR_PAGE21 if output_kind.is_executable() => {
                debug_assert!(
                    section_bytes[offset..].starts_with(TLSDESC_ADR_PAGE21_INSN_SEQUENCE),
                    "Unknown R_AARCH64_TLSDESC_ADR_PAGE21 instruction"
                );
                return Some(Relaxation {
                    kind: RelaxationKind::ReplaceWithNop,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_NONE),
                    mandatory: output_kind.is_static_executable(),
                });
            }
            object::elf::R_AARCH64_TLSDESC_LD64_LO12 if output_kind.is_executable() => {
                return Some(Relaxation {
                    kind: RelaxationKind::ReplaceWithNop,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_NONE),
                    mandatory: output_kind.is_static_executable(),
                });
            }
            object::elf::R_AARCH64_TLSDESC_ADD_LO12 if output_kind.is_executable() => {
                debug_assert!(
                    section_bytes[offset..].starts_with(TLSDESC_ADD_LO12_INSN_SEQUENCE),
                    "Unknown R_AARCH64_TLSDESC_ADD_LO12 instruction"
                );
                return Some(Relaxation {
                    kind: RelaxationKind::AdrpX0,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_TLSIE_ADR_GOTTPREL_PAGE21),
                    mandatory: output_kind.is_static_executable(),
                });
            }
            object::elf::R_AARCH64_TLSDESC_CALL if output_kind.is_executable() => {
                return Some(Relaxation {
                    kind: RelaxationKind::LdrX0,
                    rel_info: rel_info_from_type!(
                        object::elf::R_AARCH64_TLSIE_LD64_GOTTPREL_LO12_NC
                    ),
                    mandatory: output_kind.is_static_executable(),
                });
            }

            // Relax local exec to init exec
            object::elf::R_AARCH64_TLSIE_ADR_GOTTPREL_PAGE21
                if output_kind.is_executable() && !interposable =>
            {
                return Some(Relaxation {
                    kind: RelaxationKind::MovzXnLsl16,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_TLSLE_MOVW_TPREL_G1),
                    mandatory: false,
                });
            }
            object::elf::R_AARCH64_TLSIE_LD64_GOTTPREL_LO12_NC
                if output_kind.is_executable() && !interposable =>
            {
                return Some(Relaxation {
                    kind: RelaxationKind::MovkXn,
                    rel_info: rel_info_from_type!(object::elf::R_AARCH64_TLSLE_MOVW_TPREL_G0_NC),
                    mandatory: false,
                });
            }
            // Relax ADRP+ADD to NOP+ADR. Applied at ADD position when previous relocation
            // was ADRP for the same symbol at the consecutive offset.
            object::elf::R_AARCH64_ADD_ABS_LO12_NC
                if !interposable && flags.has_link_time_address() && sym_addr != 0 =>
            {
                if let Some(prev) = previous_relocation
                    && prev.kind == object::elf::R_AARCH64_ADR_PREL_PG_HI21
                    && prev.offset + 4 == offset_in_section
                    && prev.addend == 0
                    && rel_addend == 0
                {
                    let offset = offset_in_section as usize;
                    if let Some(adrp_bytes) = offset
                        .checked_sub(4)
                        .and_then(|s| section_bytes.get(s..offset))
                        && let Some(add_bytes) = section_bytes.get(offset..offset + 4)
                    {
                        let adrp_instr =
                            u64::from(u32::from_le_bytes(adrp_bytes.try_into().unwrap()));
                        let add_instr =
                            u64::from(u32::from_le_bytes(add_bytes.try_into().unwrap()));
                        if (adrp_instr as u32 & 0x9f000000) == 0x90000000
                            && (add_instr as u32 & 0xffc00000) == 0x91000000
                            && adrp_instr.extract_bit_range(0..5)
                                == add_instr.extract_bit_range(0..5)
                            && adrp_instr.extract_bit_range(0..5)
                                == add_instr.extract_bit_range(5..10)
                        {
                            let add_place = section_address + offset_in_section;
                            let diff = (sym_addr as i64)
                                .wrapping_add(rel_addend)
                                .wrapping_sub(add_place as i64);
                            if AllowedRange::from_bit_size(21, Sign::Signed).contains(diff) {
                                return Some(Relaxation {
                                    kind: RelaxationKind::AdrpAddToNopAdr,
                                    rel_info: rel_info_from_type!(
                                        object::elf::R_AARCH64_ADR_PREL_LO21
                                    ),
                                    mandatory: false,
                                });
                            }
                        }
                    }
                }
            }
            _ => (),
        }

        None
    }

    fn is_symbol_variant_pcs(
        object: &<Self::Platform as Platform>::File<'_>,
        symbol_index: object::SymbolIndex,
    ) -> bool {
        object
            .symbol(symbol_index)
            .is_ok_and(|sym| sym.st_other.contains(object::elf::STO_AARCH64_VARIANT_PCS))
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

    fn thunk_config() -> Option<crate::platform::ThunkConfig> {
        Some(crate::platform::ThunkConfig {
            primary_function_part_id: const {
                output_section_id::TEXT.part_id_with_alignment::<Elf64>(Alignment { exponent: 2 })
            },
            min_branch_range: MIN_BRANCH_RANGE,
            thunk_size: THUNK_TEMPLATE.len() as u64,
        })
    }

    fn write_thunk(thunk_address: u64, target_address: u64, buf: &mut [u8]) {
        // Use ADRP+ADD+BR - a PC-relative thunk that works in PIE binaries.
        buf.copy_from_slice(THUNK_TEMPLATE);

        let thunk_page = thunk_address & !PAGE_MASK_4KB;
        let target_page = target_address & !PAGE_MASK_4KB;
        let page_diff = (target_page as i64).wrapping_sub(thunk_page as i64);
        let page_count = (page_diff / SIZE_4KB as i64) as u64 & 0x1F_FFFF;
        AArch64Instruction::Adr.write_to_value(page_count, false, &mut buf[0..4]);

        AArch64Instruction::Add.write_to_value(
            target_address & PAGE_MASK_4KB,
            false,
            &mut buf[4..8],
        );
    }

    /// When BTI is enforced, a thunk reaches its target with an indirect `br`, so targets that
    /// don't already start with a landing pad get `bti c; b target` written just before them.
    fn write_thunk_landing_pads<'data>(
        object: &crate::layout::ObjectLayout<'data, Elf64>,
        buffers: &mut crate::output_section_part_map::OutputSectionPartMap<&mut [u8]>,
        layout: &Layout<'data, Elf64>,
        section_index: object::SectionIndex,
    ) -> Result {
        if layout.args().only_keep_debug() {
            return Ok(());
        }

        for pad in layout
            .thunk_landing_pads
            .pads_before_section(object.file_id, section_index.0)
        {
            let &pad_address = layout
                .thunk_landing_pad_addresses
                .get(&pad.target)
                .with_context(|| format!("Missing BTI pad address for {}", pad.target))?;

            let resolution = layout
                .merged_symbol_resolution(pad.target)
                .with_context(|| format!("Missing BTI target resolution for {}", pad.target))?;

            let target_address = resolution.raw_value;
            // The `b` is the second instruction of the pad.
            let branch_address = pad_address + 4;
            let displacement = target_address.wrapping_sub(branch_address);

            ensure!(
                displacement.is_multiple_of(4)
                    && AllowedRange::from_bit_size(28, Sign::Signed).contains(displacement as i64),
                "BTI landing pad branch from {branch_address:#x} to {target_address:#x} is out of range"
            );

            let branch = crate::erratum843419::encode_branch(branch_address, target_address)?;

            let buf = buffers
                .get_mut(pad.part_id)
                .split_off_mut(..pad.size as usize)
                .ok_or_else(|| crate::file_writer::insufficient_allocation("BTI landing pad"))?;

            // Any alignment padding after the two instructions is zero (`udf`) and never executed.
            buf.fill(0);
            buf[..4].copy_from_slice(&BTI_C.to_le_bytes());
            buf[4..8].copy_from_slice(&branch.to_le_bytes());
        }

        Ok(())
    }

    /// When BTI is enforced, a thunk reaches its target with an indirect `br`, so every thunk
    /// target that doesn't already start with a landing pad gets one.
    fn allocate_thunk_landing_pads<'data>(
        thunk_blocks: &[crate::thunks::ThunkBlock],
        group_states: &mut [crate::layout::GroupState<Elf64>],
        total_sizes: &mut crate::output_section_part_map::OutputSectionPartMap<u64>,
        symbol_db: &crate::symbol_db::SymbolDb<Elf64>,
        per_symbol_flags: &crate::value_flags::PerSymbolFlags,
        output_sections: &crate::output_section_id::OutputSections<Elf64>,
        format_specific: &<Elf64 as Platform>::FinaliseSizesExt<'data>,
    ) -> Result<crate::thunks::LandingPadPlan> {
        if !format_specific.has_aarch64_bti() {
            return Ok(crate::thunks::LandingPadPlan::default());
        }

        let plan = collect_bti_pad_targets(
            thunk_blocks,
            group_states,
            symbol_db,
            per_symbol_flags,
            output_sections,
        )?;
        reserve_bti_pad_space(&plan, group_states, total_sizes);
        Ok(plan)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Relaxation {
    kind: RelaxationKind,
    rel_info: RelocationKindInfo,
    mandatory: bool,
}

const TLSDESC_ADR_PAGE21_INSN_SEQUENCE: &[u8] = &[
    0x0, 0x0, 0x0, 0x90, // adrp    x0, 0
];

const TLSDESC_ADD_LO12_INSN_SEQUENCE: &[u8] = &[
    0x0, 0x0, 0x0, 0x91, // add     x0, x0, #0x0
];

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
