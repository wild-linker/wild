//! A faster way to apply the relocations in a debug section's RELA table.
//!
//! Almost all DWARF relocations are absolute 32- or 64-bit stores against a few hot symbols, such
//! as the section symbols of `.debug_str` and `.debug_line`. This module decides once per symbol
//! how its relocations get their value, and once per relocation type how to write it, so that the
//! common case is an add and a store. Everything else goes through [`apply_debug_relocation`],
//! which keeps relocation semantics and error messages in one place.
//!
//! Large sections are also applied in parallel shards without first checking that their
//! relocations are sorted. See [`try_apply_speculatively`] for why that's safe on every
//! architecture.

use super::DebugRelocationShardRange;
use super::ElfLayout;
use super::PARALLEL_DEBUG_RELOCATION_MIN;
use super::RelocationCache;
use super::apply_debug_rela_relocations;
use super::apply_debug_relocation;
use super::debug_tombstone_value;
use super::display_relocation;
use super::record_debug_relocations;
use crate::bail;
use crate::elf;
use crate::elf::ElfClass;
use crate::error::Context as _;
use crate::error::Result;
use crate::layout::ObjectLayout;
use crate::output_section_id::OutputSectionId;
use crate::platform::Arch;
use crate::platform::ObjectFile as _;
use crate::platform::Relocation as _;
use crate::platform::Symbol as _;
use crate::resolution::SectionSlot;
use crate::string_merging::StringMergeSectionSlot;
use crate::string_merging::merged_string_address;
use linker_utils::elf::AllowedRange;
use linker_utils::elf::RelocationKind;
use linker_utils::elf::RelocationKindInfo;
use linker_utils::elf::RelocationSize;
use linker_utils::relaxation::opt_input_to_output;
use rayon::iter::IntoParallelIterator as _;
use rayon::iter::ParallelIterator as _;

/// Applies the relocations of a debug section that uses a RELA table, with the same result and
/// errors as `apply_debug_rela_relocations`.
pub(super) fn apply_rela<'data, C: ElfClass, A: Arch<Platform = elf::Elf<C>>>(
    object: &ObjectLayout<'data, elf::Elf<C>>,
    out: &mut [u8],
    section_index: object::SectionIndex,
    relocations: &[elf::Rela<C>],
    layout: &ElfLayout<'data, C>,
) -> Result {
    let tombstone_value = debug_tombstone_value(object.object.section_name(section_index)?);
    let shard_count =
        rayon::current_num_threads().min(relocations.len().div_ceil(PARALLEL_DEBUG_RELOCATION_MIN));

    if relocations.len() >= PARALLEL_DEBUG_RELOCATION_MIN && shard_count >= 2 {
        // Restoring the input bytes below doesn't reproduce relaxation.
        let speculation = if object.section_relax_deltas.get(section_index.0).is_some() {
            Speculation::NotAttempted
        } else {
            try_apply_speculatively::<C, A>(
                object,
                out,
                relocations,
                layout,
                tombstone_value,
                shard_count,
            )
        };
        match speculation {
            Speculation::Applied => {
                record_debug_relocations(object, section_index, layout, relocations.len());
                return Ok(());
            }
            Speculation::Failed => {
                // Some shards may have written already, so start again from the input bytes.
                let section = object.object.section(section_index)?;
                object.object.copy_section_data(section, out)?;
            }
            Speculation::NotAttempted => {}
        }
        return apply_debug_rela_relocations::<C, A>(
            object,
            out,
            section_index,
            relocations,
            layout,
        );
    }

    apply_in_order::<C, A, false>(object, out, relocations, layout, tombstone_value, 0, None)?;
    record_debug_relocations(object, section_index, layout, relocations.len());
    Ok(())
}

enum Speculation {
    Applied,
    /// Nothing was written.
    NotAttempted,
    /// Some output may have been written and must be restored.
    Failed,
}

/// Applies relocations in parallel shards without first checking that they're sorted.
///
/// Each shard owns the output from its first relocation's offset up to the next shard's first
/// offset. A shard stops at the first relocation that falls outside that range, that fails, or
/// that isn't a pure overwrite, i.e. whose value depends on the bytes it replaces or on the
/// previous relocation, like the add/subtract and ULEB128 pairs of RISC-V and LoongArch.
///
/// If every shard finishes, each byte was written only by relocations of one shard, in their
/// original order, with values that don't depend on each other: the serial result. Otherwise the
/// caller restores the input bytes and applies the section with the generic path, which then also
/// reports any error.
fn try_apply_speculatively<'data, C: ElfClass, A: Arch<Platform = elf::Elf<C>>>(
    object: &ObjectLayout<'data, elf::Elf<C>>,
    out: &mut [u8],
    relocations: &[elf::Rela<C>],
    layout: &ElfLayout<'data, C>,
    tombstone_value: u64,
    shard_count: usize,
) -> Speculation {
    let Some(ranges) = speculative_shard_ranges(relocations.len(), out.len(), shard_count, |i| {
        elf::ElfRela::<C>::new(relocations[i]).offset()
    }) else {
        return Speculation::NotAttempted;
    };

    let mut remaining = out;
    let mut shards = Vec::with_capacity(ranges.len());
    for range in ranges {
        let (shard_out, rest) = remaining.split_at_mut(range.output.end - range.output.start);
        remaining = rest;
        shards.push((range.relocations, range.output.start, shard_out));
    }

    let all_applied = shards
        .into_par_iter()
        .map(|(range, output_offset, shard_out)| {
            let previous = range
                .start
                .checked_sub(1)
                .map(|index| elf::ElfRela::new(relocations[index]));
            apply_in_order::<C, A, true>(
                object,
                shard_out,
                &relocations[range],
                layout,
                tombstone_value,
                output_offset as u64,
                previous,
            )
            .is_ok()
        })
        .all(|ok| ok);
    if all_applied {
        Speculation::Applied
    } else {
        Speculation::Failed
    }
}

/// Splits relocations into `shard_count` equal runs by index. Each shard owns the output from its
/// first relocation's offset up to the next shard's first offset (the first shard starts at zero).
/// Returns None if those offsets go backwards or past the end of the output. Unlike the
/// pre-checked split, this reads one offset per shard; whether every relocation fits its shard is
/// checked as it's applied.
fn speculative_shard_ranges(
    relocation_count: usize,
    output_len: usize,
    shard_count: usize,
    offset_at: impl Fn(usize) -> u64,
) -> Option<Vec<DebugRelocationShardRange>> {
    let shard_count = shard_count.min(relocation_count);
    if shard_count < 2 {
        return None;
    }
    let start_of = |shard: usize| relocation_count * shard / shard_count;
    let mut ranges = Vec::with_capacity(shard_count);
    let mut output_start = 0;
    for shard in 0..shard_count {
        let relocations = start_of(shard)..start_of(shard + 1);
        let output_end = if shard + 1 == shard_count {
            output_len
        } else {
            usize::try_from(offset_at(relocations.end)).ok()?
        };
        if output_start > output_end || output_end > output_len {
            return None;
        }
        ranges.push(DebugRelocationShardRange {
            relocations,
            output: output_start..output_end,
        });
        output_start = output_end;
    }
    Some(ranges)
}

/// Applies `relocations` in order to `out`, which starts at `output_offset` within the section.
/// Relocations that the fast path doesn't handle go to [`apply_debug_relocation`]. When
/// `SPECULATIVE` is set, this instead fails on anything that isn't a pure overwrite within `out`.
fn apply_in_order<'data, C: ElfClass, A: Arch<Platform = elf::Elf<C>>, const SPECULATIVE: bool>(
    object: &ObjectLayout<'data, elf::Elf<C>>,
    out: &mut [u8],
    relocations: &[elf::Rela<C>],
    layout: &ElfLayout<'data, C>,
    tombstone_value: u64,
    output_offset: u64,
    previous: Option<elf::ElfRela<C>>,
) -> Result {
    let mut relocation_cache = RelocationCache {
        previous,
        ..Default::default()
    };
    let mut fast = FastPath::default();

    for &rel in relocations {
        let rel = elf::ElfRela::<C>::new(rel);
        let offset_in_section = rel.offset();
        let shard_offset = offset_in_section
            .checked_sub(output_offset)
            .context("Debug relocation precedes its output shard")?;
        if !fast.try_apply::<C, A>(object, shard_offset, &rel, layout, tombstone_value, out) {
            if SPECULATIVE
                && !(shard_offset <= out.len() as u64 && is_pure_overwrite::<C, A>(rel.raw_type()))
            {
                bail!("Debug relocation can't be applied speculatively");
            }
            apply_debug_relocation::<C, A, _>(
                object,
                shard_offset,
                &rel,
                layout,
                tombstone_value,
                out,
                &relocation_cache,
            )
            .with_context(|| {
                format!(
                    "Failed to apply {} at offset 0x{offset_in_section:x}",
                    display_relocation::<C, A, _>(object, &rel, layout)
                )
            })?;
        }
        relocation_cache.previous = Some(rel);
    }
    Ok(())
}

/// Whether relocations of this type write a value that depends on neither the bytes they
/// overwrite nor the previous relocation. Only such relocations may be applied speculatively.
fn is_pure_overwrite<C: ElfClass, A: Arch<Platform = elf::Elf<C>>>(
    r_type: object::elf::RelocationType,
) -> bool {
    A::relocation_from_raw(r_type).is_ok_and(|info| {
        matches!(
            info.kind,
            RelocationKind::Absolute | RelocationKind::AbsoluteSet | RelocationKind::DtpOff
        ) && matches!(info.size, RelocationSize::ByteSize(_))
    })
}

/// Per-run caches for the fast path.
#[derive(Default)]
struct FastPath {
    symbols: SymbolCache,
    writes: WriteCache,
}

impl FastPath {
    /// Applies an absolute relocation whose value is a base plus addend, a tombstone or an
    /// unnamed merged string. Returns false, having written nothing, for anything else or
    /// anything that fails, which [`apply_debug_relocation`] then handles.
    #[inline(always)]
    fn try_apply<'data, C: ElfClass, A: Arch<Platform = elf::Elf<C>>>(
        &mut self,
        object: &ObjectLayout<'data, elf::Elf<C>>,
        offset_in_section: u64,
        rel: &elf::ElfRela<C>,
        layout: &ElfLayout<'data, C>,
        tombstone_value: u64,
        out: &mut [u8],
    ) -> bool {
        let Some(symbol_index) = rel.symbol() else {
            return false;
        };
        let Some(write) = self.writes.get::<C, A>(rel.raw_type()) else {
            return false;
        };
        let Ok(symbol_value) = self.symbols.get_or_insert_with(symbol_index, || {
            SymbolValue::new(object, symbol_index, layout)
        }) else {
            return false;
        };
        let value = match symbol_value {
            SymbolValue::Base(base) => base.wrapping_add(rel.addend() as u64),
            SymbolValue::Tombstone => tombstone_value,
            SymbolValue::MergedString(target) => {
                let Ok(address) = merged_string_address(
                    target.merge_slot,
                    target.section_id,
                    target.symbol_value.wrapping_add(rel.addend() as u64),
                    &layout.merged_strings,
                    &layout.merged_string_start_addresses,
                ) else {
                    return false;
                };
                address
            }
            SymbolValue::Slow => return false,
        };
        let Some(place) = out.get_mut(offset_in_section as usize..) else {
            return false;
        };
        write.write(value, place)
    }
}

/// How absolute relocations against a symbol get their value, decided once per symbol.
#[derive(Clone, Copy, Debug)]
enum SymbolValue {
    /// The value is this base plus the addend.
    Base(u64),
    /// The symbol's section was discarded or never loaded, so the value is the tombstone.
    Tombstone,
    /// An unnamed symbol in a string-merge section, such as `.debug_str`'s section symbol. The
    /// addend picks the string.
    MergedString(MergedStringTarget),
    /// Anything else, such as ifuncs or anything that might report an error.
    Slow,
}

#[derive(Clone, Copy, Debug)]
struct MergedStringTarget {
    merge_slot: StringMergeSectionSlot,
    section_id: OutputSectionId,
    symbol_value: u64,
}

impl SymbolValue {
    /// Mirrors how [`apply_debug_relocation`] computes the value of an absolute relocation, and
    /// returns [`SymbolValue::Slow`] wherever that could do anything other than one of the cases
    /// above.
    fn new<'data, C: ElfClass>(
        object: &ObjectLayout<'data, elf::Elf<C>>,
        symbol_index: object::SymbolIndex,
        layout: &ElfLayout<'data, C>,
    ) -> Result<Self> {
        let sym = object.object.symbol(symbol_index)?;
        let section_index = object.object.symbol_section(sym, symbol_index)?;

        let raw_value = match layout
            .merged_symbol_resolution(object.symbol_id_range.input_to_id(symbol_index))
        {
            Some(resolution) if resolution.flags.is_ifunc() => return Ok(Self::Slow),
            Some(resolution) => Some(resolution.raw_value),
            None => section_index.and_then(|section_index| {
                let section_address = object.section_resolutions[section_index.0].address()?;
                Some(
                    section_address
                        + opt_input_to_output(
                            object.section_relax_deltas.get(section_index.0),
                            sym.value(),
                        ),
                )
            }),
        };

        let Some(section_index) = section_index else {
            return Ok(raw_value.map_or(Self::Tombstone, Self::Base));
        };
        let slot = &object.sections[section_index.0];
        let merged_string = || -> Self {
            match slot {
                SectionSlot::MergeStrings(merge_slot) if !sym.has_name() => {
                    let input_section_id = object.section_id_range.input_to_id(section_index);
                    Self::MergedString(MergedStringTarget {
                        merge_slot: *merge_slot,
                        section_id: layout.symbol_db.section_part_ids[input_section_id.as_usize()]
                            .output_section_id::<elf::Elf<C>>(),
                        symbol_value: sym.value(),
                    })
                }
                _ => Self::Slow,
            }
        };
        Ok(match (raw_value, slot) {
            // `Resolution::value_with_addend` looks up a merged string when the value is zero.
            (Some(0), SectionSlot::MergeStrings(..)) => merged_string(),
            (Some(raw_value), _) => Self::Base(raw_value),
            (None, SectionSlot::MergeStrings(..)) => merged_string(),
            (None, SectionSlot::Discard | SectionSlot::Unloaded(..)) => Self::Tombstone,
            (None, _) => Self::Slow,
        })
    }
}

/// Number of entries in [`SymbolCache`]. A power of two, so the slot is the low bits of the
/// symbol index.
const SYMBOL_CACHE_SLOTS: usize = 32;

/// A direct-mapped cache of [`SymbolValue`]s. `.debug_info` interleaves references to a few hot
/// symbols with references to code, so a single "previous symbol" entry misses about half the
/// time, while a few dozen slots catch nearly all of the repeats. The slots live on the stack,
/// because a cache is created for every debug section and most sections are small.
struct SymbolCache {
    slots: [Option<(object::SymbolIndex, SymbolValue)>; SYMBOL_CACHE_SLOTS],
}

impl Default for SymbolCache {
    fn default() -> Self {
        Self {
            slots: [None; SYMBOL_CACHE_SLOTS],
        }
    }
}

impl SymbolCache {
    #[inline(always)]
    fn get_or_insert_with(
        &mut self,
        symbol_index: object::SymbolIndex,
        resolve: impl FnOnce() -> Result<SymbolValue>,
    ) -> Result<SymbolValue> {
        let slot = &mut self.slots[symbol_index.0 % SYMBOL_CACHE_SLOTS];
        if let Some((cached_index, value)) = *slot
            && cached_index == symbol_index
        {
            return Ok(value);
        }
        let value = resolve()?;
        *slot = Some((symbol_index, value));
        Ok(value)
    }
}

/// How to write an absolute relocation of one type: a little-endian store of `size` bytes after
/// a range check, extracted from its [`RelocationKindInfo`].
#[derive(Clone, Copy)]
struct AbsoluteWrite {
    r_type: object::elf::RelocationType,
    size: u8,
    range: AllowedRange,
}

impl AbsoluteWrite {
    /// Returns None for relocation types that need more than a plain store.
    fn new(r_type: object::elf::RelocationType, info: &RelocationKindInfo) -> Option<Self> {
        let RelocationSize::ByteSize(size @ (4 | 8)) = info.size else {
            return None;
        };
        (info.kind == RelocationKind::Absolute && info.alignment == 1).then_some(Self {
            r_type,
            size,
            range: info.range,
        })
    }

    /// Returns false, having written nothing, if the value is out of range or `out` is too short.
    #[inline(always)]
    fn write(self, value: u64, out: &mut [u8]) -> bool {
        if !self.range.contains(value as i64) {
            return false;
        }
        // Fixed-size arms, so that each is a single store rather than a variable-length copy.
        let written = match self.size {
            4 => out
                .first_chunk_mut()
                .map(|place: &mut [u8; 4]| *place = (value as u32).to_le_bytes()),
            8 => out
                .first_chunk_mut()
                .map(|place: &mut [u8; 8]| *place = value.to_le_bytes()),
            _ => None,
        };
        written.is_some()
    }
}

/// Recently used [`AbsoluteWrite`]s, indexed by the low bit of the relocation type. DWARF
/// alternates between a 32-bit and a 64-bit absolute type, and on x86-64, AArch64 and RISC-V
/// those have different low bits, so both stay cached. Entries are checked against the type, so
/// types that share a slot only cost misses.
#[derive(Default)]
struct WriteCache {
    slots: [Option<AbsoluteWrite>; 2],
}

impl WriteCache {
    #[inline(always)]
    fn get<C: ElfClass, A: Arch<Platform = elf::Elf<C>>>(
        &mut self,
        r_type: object::elf::RelocationType,
    ) -> Option<AbsoluteWrite> {
        let slot = &mut self.slots[(r_type.0 & 1) as usize];
        match *slot {
            Some(write) if write.r_type == r_type => Some(write),
            _ => {
                let write = A::relocation_from_raw(r_type)
                    .ok()
                    .and_then(|info| AbsoluteWrite::new(r_type, &info))?;
                *slot = Some(write);
                Some(write)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speculative_shards_split_output_at_shard_start_offsets() {
        let offsets = [0, 4, 8, 12, 16, 20, 24, 28];
        let ranges = speculative_shard_ranges(offsets.len(), 32, 4, |i| offsets[i]).unwrap();
        let expected = [(0..2, 0..8), (2..4, 8..16), (4..6, 16..24), (6..8, 24..32)];
        assert_eq!(
            ranges,
            expected
                .into_iter()
                .map(|(relocations, output)| DebugRelocationShardRange {
                    relocations,
                    output
                })
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn speculative_shards_reject_backwards_or_out_of_bounds_starts() {
        // Shard starts go backwards.
        let offsets = [0, 4, 20, 24, 8, 12, 28, 30];
        assert!(speculative_shard_ranges(offsets.len(), 32, 4, |i| offsets[i]).is_none());
        // A shard starts past the end of the output.
        let offsets = [0, 4, 8, 12, 40, 44, 48, 52];
        assert!(speculative_shard_ranges(offsets.len(), 32, 4, |i| offsets[i]).is_none());
        // Too few relocations to split.
        assert!(speculative_shard_ranges(1, 32, 4, |_| 0).is_none());
    }

    #[test]
    fn only_pure_overwrites_are_speculative() {
        use crate::elf::Class64;
        use crate::elf_aarch64::ElfAArch64;
        use crate::elf_loongarch64::ElfLoongArch64;
        use crate::elf_riscv64::ElfRiscV64;
        use crate::elf_x86_64::ElfX86_64;
        use object::elf::*;

        assert!(is_pure_overwrite::<Class64, ElfX86_64>(R_X86_64_32));
        assert!(is_pure_overwrite::<Class64, ElfX86_64>(R_X86_64_64));
        assert!(is_pure_overwrite::<Class64, ElfX86_64>(R_X86_64_DTPOFF32));
        assert!(is_pure_overwrite::<Class64, ElfAArch64>(R_AARCH64_ABS64));
        assert!(is_pure_overwrite::<Class64, ElfRiscV64>(R_RISCV_64));
        // Relocations that read the bytes they overwrite, or the previous relocation.
        for r_type in [
            R_RISCV_ADD32,
            R_RISCV_SUB32,
            R_RISCV_SET6,
            R_RISCV_SUB6,
            R_RISCV_SET_ULEB128,
            R_RISCV_SUB_ULEB128,
        ] {
            assert!(
                !is_pure_overwrite::<Class64, ElfRiscV64>(r_type),
                "{r_type:?}"
            );
        }
        for r_type in [
            R_LARCH_ADD32,
            R_LARCH_SUB32,
            R_LARCH_ADD6,
            R_LARCH_SUB6,
            R_LARCH_ADD_ULEB128,
            R_LARCH_SUB_ULEB128,
        ] {
            assert!(
                !is_pure_overwrite::<Class64, ElfLoongArch64>(r_type),
                "{r_type:?}"
            );
        }
    }

    #[test]
    fn absolute_write_checks_range_and_bounds() {
        let r_type = object::elf::R_X86_64_32;
        let info = crate::elf_x86_64::ElfX86_64::relocation_from_raw(r_type).unwrap();
        let write = AbsoluteWrite::new(r_type, &info).unwrap();
        let mut out = [0xaa; 6];
        assert!(write.write(0x1122_3344, &mut out));
        assert_eq!(out, [0x44, 0x33, 0x22, 0x11, 0xaa, 0xaa]);
        // Out of range for an unsigned 32-bit relocation.
        assert!(!write.write(1 << 32, &mut out));
        // Too short.
        assert!(!write.write(1, &mut out[3..]));
        assert_eq!(out, [0x44, 0x33, 0x22, 0x11, 0xaa, 0xaa]);
    }

    #[test]
    fn symbol_cache_reuses_recent_symbols() {
        let resolve_count = std::cell::Cell::new(0);
        let mut cache = SymbolCache::default();
        let mut get = |symbol_index: usize| {
            let value = cache
                .get_or_insert_with(object::SymbolIndex(symbol_index), || {
                    resolve_count.set(resolve_count.get() + 1);
                    Ok(SymbolValue::Base(symbol_index as u64))
                })
                .unwrap();
            let SymbolValue::Base(base) = value else {
                panic!("Unexpected cached value {value:?}");
            };
            base
        };

        assert_eq!(get(7), 7);
        assert_eq!(get(7), 7);
        assert_eq!(get(8), 8);
        // 7 and 8 occupy different slots, so returning to 7 is a hit.
        assert_eq!(get(7), 7);
        assert_eq!(resolve_count.get(), 2);

        // A symbol that maps to 7's slot evicts it.
        let colliding = 7 + SYMBOL_CACHE_SLOTS;
        assert_eq!(get(colliding), colliding as u64);
        assert_eq!(get(7), 7);
        assert_eq!(resolve_count.get(), 4);
    }
}
