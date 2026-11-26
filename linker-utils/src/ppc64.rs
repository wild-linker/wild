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
use crate::utils::u32_from_slice;

// TODO: Upstream to `object` and use constants here once available.
pub const R_PPC64_PLTSEQ: object::elf::RelocationType = object::elf::RelocationType(119);
pub const R_PPC64_PLTCALL: object::elf::RelocationType = object::elf::RelocationType(120);

/// `nop` (`ori 0, 0, 0`).
pub const INSN_NOP: u32 = 0x6000_0000;
/// `addis r3, r13, 0`.
pub const INSN_ADDIS_R3_R13: u32 = 0x3c6d_0000;
/// `addi r3, r3, 0`.
pub const INSN_ADDI_R3_R3: u32 = 0x3863_0000;
/// `add r3, r3, r13`.
pub const INSN_ADD_R3_R3_R13: u32 = 0x7c63_6a14;
/// `addi r3, r13, 0x1000` (adds `DTP - TP = 0x8000 - 0x7000` to `r13`).
pub const INSN_ADDI_R3_R13_DTP_BIAS: u32 = 0x386d_1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelaxationKind {
    /// Leave the instruction alone. Used when we only want to change the kind of relocation used.
    NoOp,

    /// Replace the instruction with `nop`.
    ReplaceWithNop,

    /// Replace with `nop` and skip the paired relocation at the same offset (used for `-fno-plt`
    /// `__tls_get_addr` call setup instructions carrying `R_PPC64_TLSGD` / `R_PPC64_TLSLD`).
    ReplaceWithNopSkipNext,

    /// Replace `addi r3, rA, 0` with `addis r3, r13, 0` (`R_PPC64_TPREL16_HA`).
    AddisR3R13,

    /// Replace `bl __tls_get_addr` / `bctrl` with `addi r3, r3, 0` (`R_PPC64_TPREL16_LO`) and skip
    /// the paired call relocation.
    AddiR3R3SkipNext,

    /// Replace `addi rD, rA, 0` with `ld rD, 0(rA)` (`R_PPC64_GOT_TPREL16_LO_DS` /
    /// `R_PPC64_GOT_TPREL16_DS`).
    AddiToLd,

    /// Replace `bl __tls_get_addr` / `bctrl` with `add r3, r3, r13` and skip the paired call
    /// relocation.
    AddR3R3R13SkipNext,

    /// Replace `bl __tls_get_addr` / `bctrl` with `addi r3, r13, 0x1000` and skip the paired call
    /// relocation.
    AddiR3R13DtpBiasSkipNext,

    /// Replace `ld rD, 0(rA)` with `addis rD, r13, 0` (`R_PPC64_TPREL16_HA`).
    LdToAddisR13,

    /// Replace an X-form TLS instruction (`add` / indexed load / indexed store with `r13`) with
    /// its D-form or DS-form equivalent using the non-`r13` register as base.
    TlsXFormToDForm,
}

/// Converts a ppc64 X-form instruction marked with `R_PPC64_TLS` (operating on `r13` and a base
/// register) into its D-form or DS-form template `(insn, is_ds_form)` with a zero displacement.
#[must_use]
pub fn tls_xform_to_dform(insn: u32) -> Option<(u32, bool)> {
    // Must be primary opcode 31 (X-form) with Rc=0.
    if (insn >> 26) != 31 || (insn & 1) != 0 {
        return None;
    }

    let rt = (insn >> 21) & 0x1f;
    let ra = (insn >> 16) & 0x1f;
    let rb = (insn >> 11) & 0x1f;
    let xo = (insn >> 1) & 0x3ff;

    // (template_opcode_and_low_xo, is_ds_form, allow_ra_is_r13)
    // Update-form loads/stores modify RA, so RA cannot be r13.
    let (template, is_ds, allow_ra_is_r13) = match xo {
        266 => (0x3800_0000, false, true),  // add   -> addi
        87 => (0x8800_0000, false, true),   // lbzx  -> lbz
        119 => (0x8c00_0000, false, false), // lbzux -> lbzu
        279 => (0xa000_0000, false, true),  // lhzx  -> lhz
        311 => (0xa400_0000, false, false), // lhzux -> lhzu
        343 => (0xa800_0000, false, true),  // lhax  -> lha
        375 => (0xac00_0000, false, false), // lhaux -> lhau
        23 => (0x8000_0000, false, true),   // lwzx  -> lwz
        55 => (0x8400_0000, false, false),  // lwzux -> lwzu
        341 => (0xe800_0002, true, true),   // lwax  -> lwa
        21 => (0xe800_0000, true, true),    // ldx   -> ld
        53 => (0xe800_0001, true, false),   // ldux  -> ldu
        215 => (0x9800_0000, false, true),  // stbx  -> stb
        247 => (0x9c00_0000, false, false), // stbux -> stbu
        407 => (0xb000_0000, false, true),  // sthx  -> sth
        439 => (0xb400_0000, false, false), // sthux -> sthu
        151 => (0x9000_0000, false, true),  // stwx  -> stw
        183 => (0x9400_0000, false, false), // stwux -> stwu
        149 => (0xf800_0000, true, true),   // stdx  -> std
        181 => (0xf800_0001, true, false),  // stdux -> stdu
        535 => (0xc000_0000, false, true),  // lfsx  -> lfs
        567 => (0xc400_0000, false, false), // lfsux -> lfsu
        599 => (0xc800_0000, false, true),  // lfdx  -> lfd
        631 => (0xcc00_0000, false, false), // lfdux -> lfdu
        663 => (0xd000_0000, false, true),  // stfsx -> stfs
        695 => (0xd400_0000, false, false), // stfsux -> stfsu
        727 => (0xd800_0000, false, true),  // stfdx -> stfd
        759 => (0xdc00_0000, false, false), // stfdux -> stfdu
        _ => return None,
    };

    let base_reg = if rb == 13 && ra != 0 {
        ra
    } else if allow_ra_is_r13 && ra == 13 && rb != 0 {
        rb
    } else {
        return None;
    };

    Some((template | (rt << 21) | (base_reg << 16), is_ds))
}

impl RelaxationKind {
    pub fn apply(self, section_bytes: &mut [u8], offset_in_section: &mut u64, _addend: &mut i64) {
        let offset = *offset_in_section as usize;
        let write_insn = |bytes: &mut [u8], insn: u32| {
            bytes[offset..offset + 4].copy_from_slice(&insn.to_le_bytes());
        };
        match self {
            RelaxationKind::NoOp => {}
            RelaxationKind::ReplaceWithNop | RelaxationKind::ReplaceWithNopSkipNext => {
                write_insn(section_bytes, INSN_NOP);
            }
            RelaxationKind::AddisR3R13 => {
                write_insn(section_bytes, INSN_ADDIS_R3_R13);
            }
            RelaxationKind::AddiR3R3SkipNext => {
                write_insn(section_bytes, INSN_ADDI_R3_R3);
            }
            RelaxationKind::AddiToLd => {
                let insn = u32_from_slice(&section_bytes[offset..offset + 4]);
                // Keep RT and RA from `addi rD, rA, 0`; rewrite opcode to `ld` (58, XO=0).
                write_insn(section_bytes, 0xe800_0000 | (insn & 0x03ff_0000));
            }
            RelaxationKind::AddR3R3R13SkipNext => {
                write_insn(section_bytes, INSN_ADD_R3_R3_R13);
            }
            RelaxationKind::AddiR3R13DtpBiasSkipNext => {
                write_insn(section_bytes, INSN_ADDI_R3_R13_DTP_BIAS);
            }
            RelaxationKind::LdToAddisR13 => {
                let insn = u32_from_slice(&section_bytes[offset..offset + 4]);
                // Keep RT from `ld rD, 0(rA)`; rewrite to `addis rD, r13, 0`.
                write_insn(section_bytes, 0x3c0d_0000 | (insn & 0x03e0_0000));
            }
            RelaxationKind::TlsXFormToDForm => {
                let insn = u32_from_slice(&section_bytes[offset..offset + 4]);
                if let Some((new_insn, _)) = tls_xform_to_dform(insn) {
                    write_insn(section_bytes, new_insn);
                }
            }
        }
    }

    #[must_use]
    pub fn next_modifier(&self) -> RelocationModifier {
        match self {
            RelaxationKind::ReplaceWithNopSkipNext
            | RelaxationKind::AddiR3R3SkipNext
            | RelaxationKind::AddR3R3R13SkipNext
            | RelaxationKind::AddiR3R13DtpBiasSkipNext => RelocationModifier::SkipNextRelocation,
            _ => RelocationModifier::Normal,
        }
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
        // TOC-relative data access. The value is `S + A - TOC`, where TOC is the start of the GOT.
        // `@ha` adds the 0x8000 high-adjust. `@h` (`TOC16_HI`) does not. `TOC16` and `TOC16_DS`
        // are the signed 16-bit `@toc` forms (the small code model) and reject values that do not
        // fit. `TOC16_DS` also requires 4-byte alignment, matching the DS instruction field.
        object::elf::R_PPC64_TOC16 => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::from_bit_size(16, Sign::Signed),
            1,
            0,
        ),
        object::elf::R_PPC64_TOC16_HI => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
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
        object::elf::R_PPC64_TOC16_DS => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::from_bit_size(16, Sign::Signed),
            4,
            0,
        ),
        object::elf::R_PPC64_TOC16_LO_DS => (
            RelocationKind::SymRelGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::Ds),
            AllowedRange::no_check(),
            4,
            0,
        ),
        object::elf::R_PPC64_PLT16_LO => (
            RelocationKind::GotRelGotBase,
            RelocationSize::bit_mask_ppc64(0, 16, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_PLT16_HI => (
            RelocationKind::GotRelGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0,
        ),
        object::elf::R_PPC64_PLT16_HA => (
            RelocationKind::GotRelGotBase,
            RelocationSize::bit_mask_ppc64(16, 32, Ppc64Instruction::D),
            AllowedRange::no_check(),
            1,
            0x8000,
        ),
        object::elf::R_PPC64_PLT16_LO_DS => (
            RelocationKind::GotRelGotBase,
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
        // TLS markers. They carry no direct fixup unless relaxed by the linker.
        object::elf::R_PPC64_TLS | object::elf::R_PPC64_TLSGD | object::elf::R_PPC64_TLSLD => (
            RelocationKind::TlsMarker,
            RelocationSize::ByteSize(0),
            AllowedRange::no_check(),
            1,
            0,
        ),
        // Non-TLS markers and no-op relocations.
        object::elf::R_PPC64_NONE | R_PPC64_PLTSEQ | R_PPC64_PLTCALL => (
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
