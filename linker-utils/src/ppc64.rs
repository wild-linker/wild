//! Relocation utilities for the ppc64 (PowerPC64 LE, ELFv2) target.
//!
//! The decoded static relocations cover absolute data, PC-relative branches and `@ha`/`@lo`
//! halves, TOC-relative data access, and the medium-model TLS sequences (local-exec, initial-exec,
//! local-dynamic, and general-dynamic). Unsupported relocation types return `None`, which causes
//! the linker to emit a clean "unsupported relocation" error rather than producing incorrect
//! output.
//!
//! The TOC base (the value of `r2` / the `.TOC.` symbol) is the start of the GOT. The conventional
//! 0x8000 bias used to centre the addressable TOC window is folded into the `@ha` relocation's
//! `bias`, so `@ha`/`@lo` pairs stay correct across the full signed 32-bit TOC range.

use crate::elf::AllowedRange;
use crate::elf::Bias;
use crate::elf::Ppc64Instruction;
use crate::elf::RelocationKind;
use crate::elf::RelocationKindInfo;
use crate::elf::RelocationSize;
use crate::elf::Sign;
use crate::relaxation::RelocationModifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelaxationKind {
    /// Leave the instruction alone. Used when we only want to change the kind of relocation used.
    NoOp,
}

impl RelaxationKind {
    pub fn apply(self, _section_bytes: &mut [u8], _offset_in_section: &mut u64, _addend: &mut i64) {
        match self {
            RelaxationKind::NoOp => {}
        }
    }

    #[must_use]
    pub fn next_modifier(&self) -> RelocationModifier {
        RelocationModifier::Normal
    }
}

#[must_use]
pub const fn relocation_type_from_raw(
    r_type: object::elf::RelocationType,
) -> Option<RelocationKindInfo> {
    let (kind, size, range, alignment, bias) = match r_type {
        // Absolute addresses.
        object::elf::R_PPC64_ADDR64 => (
            RelocationKind::Absolute,
            RelocationSize::ByteSize(8),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_ADDR32 => (
            RelocationKind::Absolute,
            RelocationSize::ByteSize(4),
            // `S + A` truncated into a 32-bit field. The overflow check mirrors binutils'
            // `complain_overflow_bitfield` for this relocation: a value is in range if its bit
            // pattern fits in 32 bits, whether read as signed or unsigned. That union is
            // `[-2^31, 2^32)` -- the negative lower bound admits a sign-extended negative addend,
            // not a negative address.
            AllowedRange::new(-(2i64.pow(31)), 2i64.pow(32)),
            1,
            0,
        ),
        // PC-relative data (e.g. .eh_frame pointers).
        object::elf::R_PPC64_REL64 => (
            RelocationKind::Relative,
            RelocationSize::ByteSize(8),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_REL32 => (
            RelocationKind::Relative,
            RelocationSize::ByteSize(4),
            AllowedRange::from_bit_size(32, Sign::Signed),
            1,
            0,
        ),
        // PC-relative branches.
        object::elf::R_PPC64_REL24 => (
            RelocationKind::Relative,
            RelocationSize::bit_mask_ppc64(0, 26, Ppc64Instruction::Branch24),
            AllowedRange::from_bit_size(26, Sign::Signed),
            4,
            0,
        ),
        object::elf::R_PPC64_REL14 => (
            RelocationKind::Relative,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Branch14),
            AllowedRange::from_bit_size(16, Sign::Signed),
            4,
            0,
        ),
        // PC-relative `@ha`/`@lo` halves (used by the TOC-pointer prologue, referencing `.TOC.`).
        object::elf::R_PPC64_REL16_HA => (
            RelocationKind::Relative,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_REL16_LO => (
            RelocationKind::Relative,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        // TOC-relative data access. The value is relative to the TOC base (start of the GOT); the
        // 0x8000 centring bias lives on the `@ha` half.
        object::elf::R_PPC64_TOC16_HA => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_TOC16_LO => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_TOC16_LO_DS => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::no_check(),
            4,
            0,
        ),
        // Local-exec. The value is `S + A - TP`, with TP = TLS start + 0x7000. `@ha` bias is the
        // high-adjust only.
        object::elf::R_PPC64_TPREL64 => (
            RelocationKind::TpOff,
            RelocationSize::ByteSize(8),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_TPREL16 => (
            RelocationKind::TpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::from_bit_size(16, Sign::Signed),
            1,
            0,
        ),
        object::elf::R_PPC64_TPREL16_LO => (
            RelocationKind::TpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_TPREL16_HI => (
            RelocationKind::TpOff,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_TPREL16_HA => (
            RelocationKind::TpOff,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_TPREL16_DS => (
            RelocationKind::TpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::from_bit_size(16, Sign::Signed),
            4,
            0,
        ),
        object::elf::R_PPC64_TPREL16_LO_DS => (
            RelocationKind::TpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::no_check(),
            4,
            0,
        ),
        // Local-dynamic offset. `S + A - DTP`, DTP = TLS start + 0x8000, is applied by
        // `dtp_offset_base`. The bias here is only the `@ha` high-adjust.
        object::elf::R_PPC64_DTPREL64 => (
            RelocationKind::DtpOff,
            RelocationSize::ByteSize(8),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_DTPREL16 => (
            RelocationKind::DtpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::from_bit_size(16, Sign::Signed),
            1,
            0,
        ),
        object::elf::R_PPC64_DTPREL16_LO => (
            RelocationKind::DtpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_DTPREL16_HI => (
            RelocationKind::DtpOff,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_DTPREL16_HA => (
            RelocationKind::DtpOff,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_DTPREL16_DS => (
            RelocationKind::DtpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::from_bit_size(16, Sign::Signed),
            4,
            0,
        ),
        object::elf::R_PPC64_DTPREL16_LO_DS => (
            RelocationKind::DtpOff,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::no_check(),
            4,
            0,
        ),
        // Initial-exec and general/local-dynamic address the GOT from `.TOC.`. `.TOC.` is the
        // start of the GOT, so these distances are from `got_base`, with the same `@ha` adjust as
        // `TOC16_*`.
        object::elf::R_PPC64_GOT_TPREL16_HA => (
            RelocationKind::GotTpOffGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_GOT_TPREL16_HI => (
            RelocationKind::GotTpOffGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TPREL16_DS => (
            RelocationKind::GotTpOffGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::from_bit_size(16, Sign::Signed),
            4,
            0,
        ),
        object::elf::R_PPC64_GOT_TPREL16_LO_DS => (
            RelocationKind::GotTpOffGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::no_check(),
            4,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSGD16 => (
            RelocationKind::TlsGdGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::from_bit_size(16, Sign::Signed),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSGD16_LO => (
            RelocationKind::TlsGdGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSGD16_HI => (
            RelocationKind::TlsGdGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSGD16_HA => (
            RelocationKind::TlsGdGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_GOT_TLSLD16 => (
            RelocationKind::TlsLdGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::from_bit_size(16, Sign::Signed),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSLD16_LO => (
            RelocationKind::TlsLdGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSLD16_HI => (
            RelocationKind::TlsLdGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_GOT_TLSLD16_HA => (
            RelocationKind::TlsLdGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        // Markers on a TLS sequence. The address calculation is a separate relocation.
        object::elf::R_PPC64_TLS | object::elf::R_PPC64_TLSGD | object::elf::R_PPC64_TLSLD => (
            RelocationKind::None,
            RelocationSize::ByteSize(0),
            AllowedRange::no_check(),
            1,
            0,
        ),
        _ => return None,
    };

    Some(RelocationKindInfo {
        kind,
        size,
        mask: None,
        range,
        alignment,
        bias: Bias::from_value(bias),
        thunkable: false,
        implicit_addend: false,
    })
}

impl Ppc64Instruction {
    /// Writes `extracted_value` (the relevant bit range of the relocation value) into the
    /// instruction field. ppc64 displacement bits map 1:1 onto instruction bit positions, so this
    /// is a masked OR with no shifting; the low bits that belong to the opcode are preserved.
    pub fn write_to_value(self, extracted_value: u64, _negative: bool, dest: &mut [u8]) {
        let field_mask: u32 = match self {
            Ppc64Instruction::D => 0x0000_ffff,
            Ppc64Instruction::Ds | Ppc64Instruction::Branch14 => 0x0000_fffc,
            Ppc64Instruction::Branch24 => 0x03ff_fffc,
        };
        let mut insn = u32::from_le_bytes([dest[0], dest[1], dest[2], dest[3]]);
        insn = (insn & !field_mask) | (extracted_value as u32 & field_mask);
        dest[0..4].copy_from_slice(&insn.to_le_bytes());
    }

    #[must_use]
    pub fn read_value(self, bytes: &[u8]) -> (u64, bool) {
        let field_mask: u32 = match self {
            Ppc64Instruction::D => 0x0000_ffff,
            Ppc64Instruction::Ds | Ppc64Instruction::Branch14 => 0x0000_fffc,
            Ppc64Instruction::Branch24 => 0x03ff_fffc,
        };
        let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        (u64::from(insn & field_mask), false)
    }
}
