//! Workarounds for the Arm Cortex-A53 errata 835769 and 843419.
//!
//! The fix, for both, is that the offending instruction gets swapped for a branch to a small stub
//! at the end of `.text`, which runs the instruction and branches back.

use crate::bail;
use crate::erratum::Erratum;
use crate::erratum::ErratumFixes;
use crate::erratum::PatchSite;
use crate::error::Result;
use crate::platform::ObjectFile;
use hashbrown::HashMap;
use object::SectionIndex;
use smallvec::SmallVec;
use std::ops::Range;

const BRANCH_RANGE: i64 = 128 * 1024 * 1024;

impl Erratum {
    const fn min_span(self) -> u64 {
        match self {
            Erratum::Mac835769 => 8,
            Erratum::Adrp843419 => 12,
        }
    }

    fn scan(self, section_index: SectionIndex, address: u64, code: Code, out: &mut Vec<PatchSite>) {
        let Some(mut offset) = self.first_candidate(address, code.start) else {
            return;
        };

        while offset + self.min_span() <= code.end {
            if let Some(offset) = self.site(code, offset) {
                out.push(PatchSite::new(section_index, offset, self));
            }

            offset = self.next_candidate(address, offset);
        }
    }

    fn first_candidate(self, address: u64, start: u64) -> Option<u64> {
        match self {
            Erratum::Mac835769 => Some(start),
            Erratum::Adrp843419 => {
                let start_address = address + start;

                start_address
                    .is_multiple_of(4)
                    .then(|| start + 0xff8_u64.saturating_sub(start_address & 0xfff))
            }
        }
    }

    fn next_candidate(self, address: u64, offset: u64) -> u64 {
        match self {
            Erratum::Mac835769 => offset + 4,
            Erratum::Adrp843419 => {
                offset
                    + if (address + offset) & 0xfff == 0xff8 {
                        4
                    } else {
                        0xffc
                    }
            }
        }
    }

    /// Offset of the instruction to patch, if a sequence starting at `offset` is affected.
    fn site(self, code: Code, offset: u64) -> Option<u64> {
        match self {
            Erratum::Mac835769 => {
                let first = code.insn(offset);
                let second = code.insn(offset + 4);

                (first.is_load_store()
                    && second.is_multiply_accumulate_64()
                    && !first.feeds(second))
                .then_some(offset + 4)
            }

            Erratum::Adrp843419 => {
                let first = code.insn(offset);
                let second = code.insn(offset + 4);
                let third = code.insn(offset + 8);

                if is_843419_sequence(first, second, third) {
                    Some(offset + 8)
                } else if code.remaining(offset) >= 16
                    && !third.is_branch()
                    && is_843419_sequence(first, second, code.insn(offset + 12))
                {
                    Some(offset + 12)
                } else {
                    None
                }
            }
        }
    }
}

/// `sections` pairs an executable input section with the address it will be placed at, in ascending
/// index order.
pub(crate) fn scan_sections<'data, F: ObjectFile<'data>>(
    fixes: ErratumFixes,
    object: &F,
    sections: &[(SectionIndex, u64)],
) -> Result<Vec<PatchSite>> {
    let mut markers = mapping_symbols(object);
    let mut sites = Vec::new();

    for &(section_index, address) in sections {
        let data = object.section_data_cow(object.section(section_index)?)?;
        let mut section_markers = markers.remove(&section_index).unwrap_or_default();

        for range in code_ranges(&mut section_markers, data.len() as u64) {
            let Some(code) = Code::new(&data, &range) else {
                continue;
            };

            for erratum in fixes {
                erratum.scan(section_index, address, code, &mut sites);
            }
        }
    }

    Ok(sites)
}

/// `patch` is [`site.patch_range()`](PatchSite::patch_range) of the object's patches, which start
/// at `patch_base`.
pub(crate) fn apply_patch(
    site: PatchSite,
    section_address: u64,
    section_out: &mut [u8],
    patch_base: u64,
    patch: &mut [u8],
) -> Result {
    let offset = site.offset as usize;
    let original = Insn::read(section_out, site.offset);
    let site_address = section_address + site.offset;
    let patch_address = patch_base + site.patch_offset();
    let resume = branch(
        patch_address + site.erratum.patch_size() - 4,
        site_address + 4,
    )?;

    match site.erratum {
        Erratum::Mac835769 => {
            Insn::NOP.write(&mut patch[0..4]);
            original.write(&mut patch[4..8]);
            resume.write(&mut patch[8..12]);
        }
        Erratum::Adrp843419 => {
            original.write(&mut patch[0..4]);
            resume.write(&mut patch[4..8]);
        }
    }

    branch(site_address, patch_address)?.write(&mut section_out[offset..offset + 4]);

    Ok(())
}

fn branch(from: u64, to: u64) -> Result<Insn> {
    let displacement = to.wrapping_sub(from) as i64;

    if displacement % 4 != 0 {
        bail!("Cortex-A53 patch branch to a misaligned address 0x{to:x}");
    }

    if !(-BRANCH_RANGE..BRANCH_RANGE).contains(&displacement) {
        bail!(
            "Cortex-A53 patch at 0x{to:x} is out of branch range of the instruction it replaces \
             at 0x{from:x}. Executable code exceeds {} MiB.",
            BRANCH_RANGE / (1024 * 1024)
        );
    }

    Ok(Insn(
        0x1400_0000 | ((displacement >> 2) as u32 & 0x03ff_ffff),
    ))
}

/// An instruction-aligned window of a section's bytes.
#[derive(Clone, Copy)]
struct Code<'data> {
    data: &'data [u8],
    start: u64,
    end: u64,
}

impl<'data> Code<'data> {
    fn new(data: &'data [u8], range: &Range<u64>) -> Option<Self> {
        let start = range.start.next_multiple_of(4);
        let end = range.end.min(data.len() as u64) & !3;

        (start < end).then_some(Code { data, start, end })
    }

    fn remaining(self, offset: u64) -> u64 {
        self.end - offset
    }

    fn insn(self, offset: u64) -> Insn {
        Insn::read(self.data, offset)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Insn(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Reg(u32);

impl Insn {
    const NOP: Insn = Insn(0xd503_201f);

    fn read(data: &[u8], offset: u64) -> Insn {
        let offset = offset as usize;
        Insn(u32::from_le_bytes(
            data[offset..offset + 4].try_into().expect("4 bytes"),
        ))
    }

    fn write(self, out: &mut [u8]) {
        out.copy_from_slice(&self.0.to_le_bytes());
    }

    const fn matches(self, mask: u32, value: u32) -> bool {
        self.0 & mask == value
    }

    const fn field(self, shift: u32, bits: u32) -> u32 {
        (self.0 >> shift) & ((1 << bits) - 1)
    }

    const fn rt(self) -> Reg {
        Reg(self.field(0, 5))
    }

    const fn rn(self) -> Reg {
        Reg(self.field(5, 5))
    }

    const fn is_adrp(self) -> bool {
        self.matches(0x9f00_0000, 0x9000_0000)
    }

    const fn is_load_store(self) -> bool {
        self.matches(0x0a00_0000, 0x0800_0000)
    }

    const fn is_load_store_exclusive(self) -> bool {
        self.matches(0x3f00_0000, 0x0800_0000)
    }

    const fn is_load_exclusive(self) -> bool {
        self.matches(0x3f40_0000, 0x0840_0000)
    }

    const fn is_load_literal(self) -> bool {
        self.matches(0x3b00_0000, 0x1800_0000)
    }

    const fn is_stnp(self) -> bool {
        self.matches(0x3bc0_0000, 0x2800_0000)
    }

    const fn is_stp_post(self) -> bool {
        self.matches(0x3bc0_0000, 0x2880_0000)
    }

    const fn is_stp_offset(self) -> bool {
        self.matches(0x3bc0_0000, 0x2900_0000)
    }

    const fn is_stp_pre(self) -> bool {
        self.matches(0x3bc0_0000, 0x2980_0000)
    }

    const fn is_stp(self) -> bool {
        self.is_stp_post() || self.is_stp_offset() || self.is_stp_pre()
    }

    const fn is_load_pair(self) -> bool {
        self.matches(0x3a40_0000, 0x2840_0000)
    }

    const fn is_ldst_unscaled(self) -> bool {
        self.matches(0x3b00_0c00, 0x3800_0000)
    }

    const fn is_ldst_imm_post(self) -> bool {
        self.matches(0x3b20_0c00, 0x3800_0400)
    }

    const fn is_ldst_unprivileged(self) -> bool {
        self.matches(0x3b20_0c00, 0x3800_0800)
    }

    const fn is_ldst_imm_pre(self) -> bool {
        self.matches(0x3b20_0c00, 0x3800_0c00)
    }

    const fn is_ldst_reg_offset(self) -> bool {
        self.matches(0x3b20_0c00, 0x3820_0800)
    }

    const fn is_ldst_reg_unsigned(self) -> bool {
        self.matches(0x3b00_0000, 0x3900_0000)
    }

    const fn is_single_register_ldst(self) -> bool {
        self.is_ldst_unscaled()
            || self.is_ldst_imm_post()
            || self.is_ldst_unprivileged()
            || self.is_ldst_imm_pre()
            || self.is_ldst_reg_offset()
            || self.is_ldst_reg_unsigned()
    }

    const fn is_st1_multiple_opcode(self) -> bool {
        matches!(self.0 & 0x0000_f000, 0x2000 | 0x6000 | 0x7000 | 0xa000)
    }

    const fn is_st1_single_opcode(self) -> bool {
        matches!(self.0 & 0x0040_e000, 0x0 | 0x4000 | 0x8000)
    }

    const fn is_st1_multiple_post(self) -> bool {
        self.matches(0xbfe0_0000, 0x0c80_0000) && self.is_st1_multiple_opcode()
    }

    const fn is_st1_single_post(self) -> bool {
        self.matches(0xbfe0_0000, 0x0d80_0000) && self.is_st1_single_opcode()
    }

    const fn is_st1(self) -> bool {
        (self.matches(0xbfff_0000, 0x0c00_0000) && self.is_st1_multiple_opcode())
            || (self.matches(0xbfff_0000, 0x0d00_0000) && self.is_st1_single_opcode())
            || self.is_st1_multiple_post()
            || self.is_st1_single_post()
    }

    const fn is_branch(self) -> bool {
        self.matches(0xfe00_0000, 0xd600_0000)
            || self.matches(0xfe00_0000, 0x5400_0000)
            || self.matches(0x7c00_0000, 0x1400_0000)
            || self.matches(0x7c00_0000, 0x3400_0000)
    }

    const fn has_writeback(self) -> bool {
        self.is_ldst_imm_pre()
            || self.is_ldst_imm_post()
            || self.is_stp_pre()
            || self.is_stp_post()
            || self.is_st1_single_post()
            || self.is_st1_multiple_post()
    }

    const fn is_load(self) -> bool {
        if self.is_load_exclusive() || self.is_load_literal() {
            return true;
        }

        if !self.is_single_register_ldst() {
            return false;
        }

        let size = self.field(30, 2);
        let vector = self.field(26, 1);
        let opc = self.field(22, 2);

        // STR of a 128-bit vector register and PRFM both have opc == 2.
        opc != 0
            && !(size == 0 && vector == 1 && opc == 2)
            && !(size == 3 && vector == 0 && opc == 2)
    }

    /// `V` set means `Rt` is a SIMD&FP register.
    const fn is_vector(self) -> bool {
        self.field(26, 1) == 1
    }

    /// A `mac` that consumes the load has to wait for it, which rules the erratum out. bfd skips
    /// the sequence too.
    const fn feeds(self, mac: Insn) -> bool {
        if self.is_vector() || !(self.is_load() || self.is_load_pair()) {
            return false;
        }

        let rt = self.rt().0;
        let rt2 = if self.is_load_pair() || (self.is_load_exclusive() && self.field(21, 1) == 1) {
            self.field(10, 5)
        } else {
            rt
        };

        let rn = mac.field(5, 5);
        let rm = mac.field(16, 5);
        let ra = mac.field(10, 5);

        rt == rn || rt == rm || rt == ra || rt2 == rn || rt2 == rm || rt2 == ra
    }

    const fn writes_to(self, reg: Reg) -> bool {
        (self.is_load() && !self.is_vector() && self.rt().0 == reg.0)
            || (self.has_writeback() && self.rn().0 == reg.0)
    }

    const fn is_multiply_accumulate_64(self) -> bool {
        self.matches(0xff00_0000, 0x9b00_0000)
            // op31 of 0b010 and 0b110 are SMULH and UMULH.
            && matches!(self.field(21, 3), 0b000 | 0b001 | 0b101)
            // Ra of XZR is a plain multiply, which can't trigger the erratum.
            && self.field(10, 5) != 31
    }
}

fn is_843419_sequence(first: Insn, second: Insn, last: Insn) -> bool {
    if !first.is_adrp() {
        return false;
    }

    let base = first.rt();

    second.is_load_store()
        && (second.is_load_store_exclusive()
            || second.is_load_literal()
            || second.is_single_register_ldst()
            || second.is_stp()
            || second.is_stnp()
            || second.is_st1())
        && !second.writes_to(base)
        && last.is_ldst_reg_unsigned()
        && last.rn() == base
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MappingKind {
    /// Ordered first so a tie at the same offset resolves to data, the safe guess.
    Data,
    Code,
}

fn mapping_symbol(name: &[u8]) -> Option<MappingKind> {
    let [b'$', kind, rest @ ..] = name else {
        return None;
    };

    if !rest.is_empty() && rest[0] != b'.' {
        return None;
    }

    match kind {
        b'x' => Some(MappingKind::Code),
        b'd' => Some(MappingKind::Data),
        _ => None,
    }
}

fn mapping_symbols<'data, F: ObjectFile<'data>>(
    object: &F,
) -> HashMap<SectionIndex, Vec<(u64, MappingKind)>> {
    let mut by_section = HashMap::<SectionIndex, Vec<(u64, MappingKind)>>::new();

    for (symbol_index, symbol) in object.enumerate_symbols() {
        let Ok(name) = object.symbol_name(symbol) else {
            continue;
        };

        let Some(kind) = mapping_symbol(name) else {
            continue;
        };

        let Ok(Some(section_index)) = object.symbol_section(symbol, symbol_index) else {
            continue;
        };

        let Ok(offset) = object.symbol_offset_in_section(symbol, section_index) else {
            continue;
        };

        by_section
            .entry(section_index)
            .or_default()
            .push((offset, kind));
    }

    by_section
}

/// No markers means all code, but anything before the first marker is data, since patching a
/// literal pool corrupts it while a missed patch only leaves the erratum in place.
fn code_ranges(
    markers: &mut Vec<(u64, MappingKind)>,
    section_size: u64,
) -> SmallVec<[Range<u64>; 4]> {
    let mut ranges = SmallVec::new();

    if markers.is_empty() {
        ranges.push(0..section_size);
        return ranges;
    }

    markers.sort_unstable();
    markers.dedup_by_key(|(offset, _)| *offset);

    let mut code_start = None;
    for &(offset, kind) in markers.iter() {
        if offset > section_size {
            break;
        }

        match (code_start, kind) {
            (None, MappingKind::Code) => code_start = Some(offset),
            (Some(start), MappingKind::Data) => {
                ranges.push(start..offset);
                code_start = None;
            }
            _ => {}
        }
    }

    if let Some(start) = code_start {
        ranges.push(start..section_size);
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADRP_X0: Insn = Insn(0x9000_0000);
    const LDR_X1_X0: Insn = Insn(0xf940_0001);
    const ADD_X0_X0: Insn = Insn(0x9100_0000);
    const B_FORWARD: Insn = Insn(0x1400_0001);

    fn scan_843419(address: u64, words: &[Insn]) -> Vec<u64> {
        let data = words
            .iter()
            .flat_map(|insn| insn.0.to_le_bytes())
            .collect::<Vec<u8>>();
        let code = Code::new(&data, &(0..data.len() as u64)).unwrap();
        let mut out = Vec::new();

        Erratum::Adrp843419.scan(SectionIndex(1), address, code, &mut out);

        out.iter().map(|site| site.offset).collect()
    }

    #[test]
    fn scans_843419_at_page_ends_only() {
        assert_eq!(scan_843419(0xff8, &[ADRP_X0, LDR_X1_X0, LDR_X1_X0]), [8]);
        assert_eq!(
            scan_843419(0xffc, &[ADRP_X0, LDR_X1_X0, ADD_X0_X0, LDR_X1_X0]),
            [12]
        );
        assert_eq!(scan_843419(0, &[ADRP_X0, LDR_X1_X0, LDR_X1_X0]), []);
        assert_eq!(
            scan_843419(0xffc, &[ADRP_X0, LDR_X1_X0, B_FORWARD, LDR_X1_X0]),
            []
        );
    }
}
