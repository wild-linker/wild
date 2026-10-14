//! Erratum 843419: A load or store might access an incorrect address.
//! Software Developers Errata Notice: https://documentation-service.arm.com/static/5fa29fddb209f547eebd361d
//!
//! Description:
//! When executing in AArch64 state, a load or store instruction which uses the result of an ADRP
//! instruction as a base register, or which uses a base register written by an instruction
//! immediately after an ADRP to the same register, might access an incorrect address.
//!
//! Workaround: Replace the final load/store in an affected sequence with a branch to a veneer
//! that executes the load/store and branches back. Keep preceding PC-relative instructions in
//! place.
//!
//! Variant 2 is intentionally excluded because it involves a dead ADRP instruction:
//! where the following instruction overwrites its destination register.

use crate::Result;
use crate::elf::ElfClass;
use crate::elf::File;
use crate::elf_aarch64::MIN_BRANCH_RANGE;
use crate::ensure;
use crate::platform::ObjectFile;
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;

const INSN_SIZE: usize = 4;
// The final load/store followed by a branch back to the original instruction stream.
const VENEER_SIZE: usize = 2 * INSN_SIZE;

const ADRP_MARK: u32 = 0x9f00_0000;
const ADRP_OPCODE: u32 = 0x9000_0000;
// LDR (unsigned offset)
const LDR_UNSIGNED_MASK: u32 = 0xffc0_0000;
const LDR_UNSIGNED_OPCODE: u32 = 0xf940_0000;
// ADD (immediate)
const ADD_IMM_MASK: u32 = 0xffc0_0000;
const ADD_IMM_OPCODE: u32 = 0x9100_0000;
// Load/store register (unsigned immediate) (ARM Manual category of instructions)
const LDR_STR_UNSIGNED_MASK: u32 = 0x3b00_0000;
const LDR_STR_UNSIGNED_OPCODE: u32 = 0x3900_0000;
const B_OPCODE: u32 = 0x1400_0000;
const NOP_OPCODE: u32 = 0xd503201f;

const REGISTER_MASK: u32 = (1 << 5) - 1;

#[derive(Debug)]
enum ArmInsn {
    Adrp { rd: u32 },
    Branch,
    Ldr { rt: u32, rn: u32 },
    Add { rd: u32 },
    LdrStr { rn: u32 },
    Unrecognized,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ErratumVariant {
    // Sequence 1 with 4 instructions
    Sequence1A,
    // Sequence 1 with 3 instructions
    Sequence1B,
}

/// An erratum variant and its section-relative ADRP byte offset.
pub(crate) type ErratumOffset = (ErratumVariant, usize);

impl ErratumVariant {
    fn instruction_count(self) -> usize {
        match self {
            Self::Sequence1A => 4,
            Self::Sequence1B => 3,
        }
    }
}

impl ArmInsn {
    fn is_final_load_store_imm(insn: &ArmInsn, register: u32) -> bool {
        match insn {
            Self::LdrStr { rn, .. } if *rn == register => true,
            Self::Ldr { rn, .. } if *rn == register => true,
            _ => false,
        }
    }

    fn is_branch(insn: u32) -> bool {
        // B, BL
        (insn & 0x7c000000) == B_OPCODE
           // B.cond and BC.cond
           || (insn & 0xff000000) == 0x54000000
           // CBZ, CBNZ
           || (insn & 0x7e000000) == 0x34000000
           // TBZ, TBNZ
           || (insn & 0x7e000000) == 0x36000000
           // BR, BLR, RET and authenticated/register control transfers
           || (insn & 0xfe000000) == 0xd6000000
    }

    fn from_opcode(insn: u32) -> Self {
        if insn & ADRP_MARK == ADRP_OPCODE {
            Self::Adrp {
                rd: insn & REGISTER_MASK,
            }
        } else if insn & LDR_UNSIGNED_MASK == LDR_UNSIGNED_OPCODE {
            // Ldr is a subset of LdrStr, so decode it first.
            Self::Ldr {
                rt: insn & REGISTER_MASK,
                rn: (insn >> 5) & REGISTER_MASK,
            }
        } else if insn & ADD_IMM_MASK == ADD_IMM_OPCODE {
            Self::Add {
                rd: insn & REGISTER_MASK,
            }
        } else if insn & LDR_STR_UNSIGNED_MASK == LDR_STR_UNSIGNED_OPCODE {
            Self::LdrStr {
                // The target register (rt) is not needed here.
                rn: (insn >> 5) & REGISTER_MASK,
            }
        } else if Self::is_branch(insn) {
            Self::Branch
        } else {
            Self::Unrecognized
        }
    }

    /// Returns true if the instruction sequence begins with an ADRP that could trigger
    /// erratum 843419.
    fn starts_with_erratum_843419(insns: &[[u8; INSN_SIZE]]) -> Option<ErratumVariant> {
        let get_insn = |data| Self::from_opcode(u32::from_le_bytes(data));

        // Apparently the following `let Adrp` is not optimized so ideally as the following check!
        if (u32::from_le_bytes(insns[0])) & ADRP_MARK != ADRP_OPCODE {
            return None;
        }

        // 1) ADRP
        let ArmInsn::Adrp { rd: register } = get_insn(insns[0]) else {
            return None;
        };

        // 2) The next instruction must not overwrite the ADRP destination register.
        // This must not write to Rn.
        match get_insn(insns[1]) {
            Self::Add { .. } | Self::Adrp { .. } | Self::Branch => return None,
            ArmInsn::Ldr { rt, .. } if rt == register => return None,
            _ => {}
        }

        let third = get_insn(insns[2]);

        // 3) Variant B
        if Self::is_final_load_store_imm(&third, register) {
            return Some(ErratumVariant::Sequence1B);
        }

        // 3) Variant A (optional 3rd instruction)
        if insns.len() >= 4 {
            // This cannot be a branch.
            // This cannot write Rn.
            match third {
                Self::Branch => {}
                Self::Add { rd, .. } if rd == register => {}
                Self::Ldr { rt, .. } if rt == register => {}
                ArmInsn::Adrp { rd } if rd == register => {}
                _ => {
                    // 4) Load/store register (unsigned immediate)" encoding class, using Rn as the
                    //    base address register.
                    if Self::is_final_load_store_imm(&get_insn(insns[3]), register) {
                        return Some(ErratumVariant::Sequence1A);
                    }
                }
            }
        }

        None
    }
}

/// The erratum depends on the low 12 bits of the instruction address.
const ERRATUM_PAGE_SIZE: usize = 4096;
/// The number of instruction positions in one erratum page.
const ERRATUM_INSN_OFFSETS: usize = ERRATUM_PAGE_SIZE / INSN_SIZE;

#[derive(Debug)]
pub(crate) struct ErratumSectionInfo {
    pub(crate) offsets: SmallVec<[ErratumOffset; 2]>,
    // Maximum padding in bytes needed to make any valid section placement safe.
    pub(crate) maximal_padding: usize,
}

/// Returns whether an ADRP offset, in instruction units, is safe from the erratum.
fn is_safe_adrp_offset(offset: usize) -> bool {
    let page_insn_offset = offset % ERRATUM_INSN_OFFSETS;
    page_insn_offset != 0xff8 / INSN_SIZE && page_insn_offset != 0xffc / INSN_SIZE
}

enum MappingSymbol {
    Code,
    Data,
}

struct SectionMappingRange {
    offset: u64,
    content: MappingSymbol,
}

/// Copies each affected sequence's final load/store into a tail slot and maps its moved address.
pub(crate) fn patch_erratum_sequences<C: ElfClass>(
    out: &mut [u8],
    section_size: usize,
    section_address: u64,
    offsets: &[ErratumOffset],
    object: &File<'_, C>,
    section_index: object::SectionIndex,
    patch_count: &AtomicU64,
) -> Result<HashMap<u64, u64>> {
    let mut mapping = HashMap::new();
    let mut mapping_symbols = None;
    let aligned_section_size = section_size.next_multiple_of(INSN_SIZE);
    ensure!(
        aligned_section_size + INSN_SIZE <= out.len(),
        "Insufficient space for branch over erratum slots"
    );
    // Fall-through must skip all reserved slots, including unused padding.
    write_branch(out, aligned_section_size, out.len())?;
    let mut tail = aligned_section_size + INSN_SIZE;
    for &(variant, offset) in offsets {
        if is_safe_adrp_offset((section_address as usize + offset) / INSN_SIZE) {
            continue;
        }
        let size = variant.instruction_count() * INSN_SIZE;

        // Only read the symbol table when we first encounter a sequence that needs patching.
        // Executable sections can contain data that happens to decode as an erratum sequence.
        if mapping_symbols.is_none() {
            let mut symbols = object
                .symbols
                .enumerate()
                .map(|(index, symbol)| -> Result<Option<SectionMappingRange>> {
                    if object.symbol_section(symbol, index)? != Some(section_index) {
                        return Ok(None);
                    }
                    let content = match object.symbol_name(symbol)? {
                        b"$x" => MappingSymbol::Code,
                        b"$d" => MappingSymbol::Data,
                        name if name.starts_with(b"$x.") => MappingSymbol::Code,
                        name if name.starts_with(b"$d.") => MappingSymbol::Data,
                        _ => return Ok(None),
                    };
                    Ok(Some(SectionMappingRange {
                        offset: object.symbol_offset_in_section(symbol, section_index)?,
                        content,
                    }))
                })
                .filter_map(Result::transpose)
                .collect::<Result<Vec<_>>>()?;
            symbols.sort_unstable_by_key(|range| range.offset);
            mapping_symbols = Some(symbols);
        }
        if !is_code_range(mapping_symbols.as_ref().unwrap(), offset, size) {
            continue;
        }
        ensure!(
            offset + size <= section_size,
            "Erratum sequence exceeds section size"
        );
        ensure!(
            tail + VENEER_SIZE <= out.len(),
            "Insufficient space for erratum slot"
        );
        // Only the final unsigned-immediate load/store moves
        let load_store_offset = offset + size - INSN_SIZE;
        out.copy_within(load_store_offset..load_store_offset + INSN_SIZE, tail);
        mapping.insert(
            section_address + load_store_offset as u64,
            section_address + tail as u64,
        );
        write_branch(out, load_store_offset, tail)?;
        write_branch(out, tail + INSN_SIZE, load_store_offset + INSN_SIZE)?;
        tail += VENEER_SIZE;

        patch_count.fetch_add(1, Relaxed);
    }

    // Fill unused tail padding with NOPs.
    for instruction in out[tail..].as_chunks_mut::<INSN_SIZE>().0 {
        *instruction = NOP_OPCODE.to_le_bytes();
    }

    Ok(mapping)
}

/// Checks that the entire half-open range is entirely covered by $x mapping symbols, not $d.
fn is_code_range(mapping_symbols: &[SectionMappingRange], offset: usize, size: usize) -> bool {
    let offset = offset as u64;
    let end = offset + size as u64;
    let index = mapping_symbols.partition_point(|range| range.offset <= offset);
    index > 0
        && matches!(mapping_symbols[index - 1].content, MappingSymbol::Code)
        && mapping_symbols[index..]
            .iter()
            .take_while(|range| range.offset < end)
            .all(|range| matches!(range.content, MappingSymbol::Code))
}

fn write_branch(out: &mut [u8], from: usize, to: usize) -> Result {
    let displacement = u32::try_from((to.wrapping_sub(from) / INSN_SIZE) & 0x03ff_ffff)?;
    out[from..from + INSN_SIZE].copy_from_slice(&(B_OPCODE | displacement).to_le_bytes());
    Ok(())
}

pub(crate) fn erratum_section_info(
    data: &[u8],
    section_alignment: u64,
) -> Result<Option<ErratumSectionInfo>> {
    debug_assert!(section_alignment.is_power_of_two());

    let insns = data.as_chunks::<INSN_SIZE>().0;

    // A busy loop where we intentionally use a plain loop as the Iterator abstraction
    // is not zero cost.
    let mut erratum_offsets: SmallVec<[_; 2]> = SmallVec::new();
    for index in 0..insns.len().saturating_sub(2) {
        if let Some(variant) = ArmInsn::starts_with_erratum_843419(&insns[index..]) {
            erratum_offsets.push((variant, index * INSN_SIZE));
        }
    }

    let section_alignment = section_alignment as usize;
    if erratum_offsets.is_empty() {
        return Ok(None);
    }

    ensure!(
        section_alignment >= INSN_SIZE,
        "unexpected small alignment: {section_alignment}"
    );
    let section_size = data.len();
    let aligned_section_size = section_size.next_multiple_of(INSN_SIZE);
    let alignment_padding = aligned_section_size - section_size;
    let mut maximal_padding = alignment_padding;

    // Take the worst case over every instruction-aligned page-relative section start,
    // independently of the section's alignment or its eventual placement.
    for section_start in 0..ERRATUM_INSN_OFFSETS {
        let veneer_count = erratum_offsets
            .iter()
            .filter(|(_, offset)| !is_safe_adrp_offset(*offset / INSN_SIZE + section_start))
            .count();
        // Reserve a branch over the veneer slots for sections that fall through, like .init/.fini.
        let padding = alignment_padding + INSN_SIZE + veneer_count * VENEER_SIZE;
        maximal_padding = maximal_padding.max(padding);
    }

    ensure!(
        (section_size + maximal_padding) as u64 <= MIN_BRANCH_RANGE,
        "Input section too large for erratum workaround code"
    );

    Ok(Some(ErratumSectionInfo {
        offsets: erratum_offsets,
        maximal_padding,
    }))
}
