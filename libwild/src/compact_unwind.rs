//! Builds a Mach-O compact unwind section.

use crate::ensure;
use crate::error::Result;
use crate::macho::ResolvedUnwindInfo;
use crate::macho::UnwindInfoWithRelocs;
use crate::symbol_db::SymbolId;
use anyhow::Context;
use hashbrown::HashMap;
use itertools::Itertools;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

// The following 3 data structures were copied from the `macho-unwind-info` crate that
// is licensed under the same licence as this project.
//
// Based on an excellent blog post that discusses the data format:
// https://gankra.github.io/blah/compact-unwinding/

// TODO: Upstream to `object` crate.

/// The `__unwind_info` header.
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy)]
#[repr(C)]
pub struct CompactUnwindInfoHeader {
    /// The version. Only version 1 is currently defined
    pub version: u32,

    /// The array of U32 global opcodes (offset relative to start of root page).
    ///
    /// These may be indexed by "compressed" second-level pages.
    pub global_opcodes_offset: u32,
    pub global_opcodes_len: u32,

    /// The array of U32 global personality codes (offset relative to start of root page).
    ///
    /// Personalities define the style of unwinding that an unwinder should use,
    /// and how to interpret the LSDA functions for a function (see below).
    pub personalities_offset: u32,
    pub personalities_len: u32,

    /// The array of [`PageEntry`]'s describing the second-level pages
    /// (offset relative to start of root page).
    pub pages_offset: u32,
    pub pages_len: u32,
    // After this point there are several dynamically-sized arrays whose precise
    // order and positioning don't matter, because they are all accessed using
    // offsets like the ones above. The arrays are:

    // global_opcodes: [u32; global_opcodes_len],
    // personalities: [u32; personalities_len],
    // pages: [PageEntry; pages_len],
    // lsdas: [LsdaEntry; unknown_len],
}

/// One element of the array of pages.
#[derive(FromBytes, IntoBytes, Immutable, Clone, Copy, Debug)]
#[repr(C)]
pub struct PageEntry {
    /// The first address mapped by this page.
    ///
    /// This is useful for binary-searching for the page that can map
    /// a specific address in the binary (the primary kind of lookup
    /// performed by an unwinder).
    pub first_address: u32,

    /// Offset of the second-level page.
    ///
    /// This may point to either a [`RegularPage`] or a [`CompressedPage`].
    /// Which it is can be determined by the 32-bit "kind" value that is at
    /// the start of both layouts.
    pub page_offset: u32,

    /// Base offset into the lsdas array that functions in this page will be
    /// relative to.
    pub lsda_index_offset: u32,
}

/// A "compressed" page.
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy)]
#[repr(C)]
pub struct CompressedPage {
    /// Always 3 (use to distinguish from RegularPage).
    pub kind: u32,

    /// The array of compressed u32 function entries (offset relative to **start of this page**).
    ///
    /// Entries are a u32 that contains two packed values (from highest to lowest bits):
    /// * 8 bits: opcode index
    ///   * 0..global_opcodes_len => index into global palette
    ///   * global_opcodes_len..255 => index into local palette (subtract global_opcodes_len)
    /// * 24 bits: instruction address
    ///   * address is relative to this page's first_address!
    pub functions_offset: u16,
    pub functions_len: u16,

    /// The array of u32 local opcodes for this page (offset relative to **start of this page**).
    pub local_opcodes_offset: u16,
    pub local_opcodes_len: u16,
}

/// LSDA entry record.
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy)]
#[repr(C)]
pub struct LsdaEntry {
    pub(crate) function_offset: u32,
    pub(crate) lsda_offset: u32,
}

const COMPRESSED_PAGE_SIZE: usize = 4096;
const COMPRESSED_PAGE_ENTRIES_COUNT: usize =
    (COMPRESSED_PAGE_SIZE - size_of::<CompressedPage>()) / size_of::<u32>();
const UNWIND_SECOND_LEVEL_COMPRESSED: u32 = 3;
// 2-bits using one-based index
const PERSONALITY_COUNT_LIMIT: usize = 3;

fn encode_personality_fn_index(encoding: u32, personality_idx: usize) -> u32 {
    debug_assert!(personality_idx < PERSONALITY_COUNT_LIMIT);
    encoding | (((personality_idx + 1) as u32) << 28)
}

pub(crate) fn output_size(unwind_info_entries: &[UnwindInfoWithRelocs]) -> Result<u64> {
    let personalities_to_idx: HashMap<SymbolId, usize> = unwind_info_entries
        .iter()
        .filter_map(|entry| entry.personality_symbol_id)
        .unique()
        .sorted()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();
    ensure!(
        personalities_to_idx.len() <= PERSONALITY_COUNT_LIMIT,
        "too many personality functions in the compact unwind: {PERSONALITY_COUNT_LIMIT}"
    );

    let encoding_values = unwind_info_entries
        .iter()
        .map(|entry| {
            let encoding = entry.entry.encoding;
            if let Some(personality) = entry.personality_symbol_id {
                let personality_idx = personalities_to_idx.get(&personality).unwrap();
                encode_personality_fn_index(encoding, *personality_idx)
            } else {
                encoding
            }
        })
        .unique()
        .count();

    // TODO: right now we only encoding index as part of the root page, based on initial
    // measurements we should be able to link large applications like Clang:
    // https://github.com/wild-linker/wild/issues/2066
    ensure!(
        u8::try_from(encoding_values).is_ok(),
        "too many encodings in compact unwind: {encoding_values}"
    );

    let lsdas = unwind_info_entries
        .iter()
        .filter_map(|entry| entry.lsda_relocation)
        .count();

    let compressed_pages = unwind_info_entries
        .len()
        .div_ceil(COMPRESSED_PAGE_ENTRIES_COUNT);
    let compressed_pages_total_size = compressed_pages * size_of::<CompressedPage>()
        + unwind_info_entries.len() * size_of::<u32>();
    // PageEntry:CompressedPage mapping is 1:1 (modulo we need one more for the termination page)
    let page_entries_total_size = (compressed_pages + 1) * size_of::<PageEntry>();
    let root_total_size = size_of::<CompactUnwindInfoHeader>()
        + encoding_values * size_of::<u32>()
        + personalities_to_idx.len() * size_of::<u32>()
        + lsdas * size_of::<LsdaEntry>();

    let size = compressed_pages_total_size + page_entries_total_size + root_total_size;
    let size =
        u32::try_from(size).with_context(|| "too large compact unwind section size: {size}")?;
    Ok(u64::from(size))
}

pub(crate) fn build(
    text_segment_start: u64,
    section_size: usize,
    unwind_info_entries: &[ResolvedUnwindInfo],
) -> Result<Vec<u8>> {
    if unwind_info_entries.is_empty() {
        return Ok(Vec::new());
    }

    let personalities = unwind_info_entries
        .iter()
        .filter_map(|entry| {
            entry.personality_symbol_id.map(|personality_symbol_id| {
                (personality_symbol_id, entry.personality_address.unwrap())
            })
        })
        .sorted()
        .unique()
        .collect_vec();
    let personalities_to_idx: HashMap<SymbolId, usize> = personalities
        .iter()
        .enumerate()
        .map(|(i, (symbol_id, _))| (*symbol_id, i))
        .collect();

    let unwind_info_entries = unwind_info_entries
        .iter()
        .sorted_by_key(|e| e.start_address)
        .collect_vec();
    let last_entry = unwind_info_entries.last().unwrap();
    let end_address = last_entry.start_address + u64::from(last_entry.entry.length);
    let lsdas = unwind_info_entries
        .iter()
        .filter_map(|e| {
            e.lsda_address.map(|lsda_address| {
                Ok(LsdaEntry {
                    function_offset: u32::try_from(e.start_address - text_segment_start)
                        .context("function offset cannot bit into u32")?,
                    lsda_offset: u32::try_from(lsda_address - text_segment_start)
                        .context("function offset cannot bit into u32")?,
                })
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let encoding_values = unwind_info_entries
        .iter()
        .map(|entry| {
            let encoding = entry.entry.encoding;
            if let Some(personality) = entry.personality_symbol_id {
                let personality_idx = personalities_to_idx.get(&personality).unwrap();
                encode_personality_fn_index(encoding, *personality_idx)
            } else {
                encoding
            }
        })
        .unique()
        .collect_vec();
    let encoding_to_index: HashMap<u32, usize> = encoding_values
        .iter()
        .enumerate()
        .map(|(i, encoding)| (*encoding, i))
        .collect();

    let compressed_pages = unwind_info_entries
        .len()
        .div_ceil(COMPRESSED_PAGE_ENTRIES_COUNT);

    let mut out = Vec::with_capacity(section_size);

    let header_size = size_of::<CompactUnwindInfoHeader>();
    let encoding_size = encoding_values.as_bytes().len();
    let personalities = personalities
        .iter()
        .map(|(_, address)| -> Result<u32> {
            let offset = address
                .checked_sub(text_segment_start)
                .context("personality GOT slot is before the image base")?;
            Ok(u32::try_from(offset).context("personality GOT offset exceeds 32 bits")?)
        })
        .collect::<Result<Vec<_>>>()?;
    let personalities_size = personalities.as_bytes().len();

    let header = CompactUnwindInfoHeader {
        version: 1,
        global_opcodes_offset: size_of::<CompactUnwindInfoHeader>() as u32,
        global_opcodes_len: u32::from(
            u8::try_from(encoding_values.len())
                .with_context(|| "too many encodings in compact unwind: {encoding_values}")?,
        ),
        pages_offset: (header_size + encoding_size + personalities_size) as u32,
        pages_len: (compressed_pages + 1) as u32,
        personalities_offset: (header_size + encoding_size) as u32,
        personalities_len: personalities.len() as u32,
    };
    out.extend(header.as_bytes());
    out.extend(encoding_values.as_bytes());
    out.extend(personalities.as_bytes());
    let header_size = out.len();

    // We build the second-level pages first.
    let second_level_entries = unwind_info_entries
        .chunks(COMPRESSED_PAGE_ENTRIES_COUNT)
        .map(|chunk| {
            let start = chunk.first().unwrap().start_address;
            let encoding_array = chunk
                .iter()
                .map(|entry| {
                    let mut encoding = entry.entry.encoding;
                    if let Some(personality) = entry.personality_symbol_id {
                        let personality_idx = personalities_to_idx.get(&personality).unwrap();
                        encoding = encode_personality_fn_index(encoding, *personality_idx);
                    }
                    (
                        entry.start_address - start,
                        // We already checked we don't have more than u32::MAX encoding values.
                        u8::try_from(*encoding_to_index.get(&encoding).unwrap()).unwrap(),
                    )
                })
                .collect_vec();
            let lsda_count = chunk
                .iter()
                .filter(|entry| entry.lsda_address.is_some())
                .count() as u32;
            (start, lsda_count, encoding_array)
        })
        .collect_vec();

    // Now having built second-level entries, we can build and encode the first-level entries.
    let lsda_data_offset = header_size + (compressed_pages + 1) * size_of::<PageEntry>();
    let second_level_first_offset = lsda_data_offset + lsdas.len() * size_of::<LsdaEntry>();

    let mut lsda_offset = 0;
    for (i, (start, lsda_count, _)) in second_level_entries.iter().enumerate() {
        let record = PageEntry {
            first_address: u32::try_from(*start - text_segment_start)
                .context("first address does not fit into u32")?,
            lsda_index_offset: (lsda_data_offset + lsda_offset * size_of::<LsdaEntry>()) as u32,
            page_offset: u32::try_from(second_level_first_offset + i * size_of::<CompressedPage>())
                .context("page offset does not fit into u32")?,
        };
        out.extend(record.as_bytes());
        lsda_offset += *lsda_count as usize;
    }

    // Emit termination page
    out.extend(
        PageEntry {
            first_address: u32::try_from(end_address - text_segment_start)
                .context("last address does not fit into u32")?,
            page_offset: 0,
            lsda_index_offset: (lsda_data_offset + lsda_offset * size_of::<LsdaEntry>()) as u32,
        }
        .as_bytes(),
    );

    // Place LSDA entries right after first-level pages.
    out.extend(lsdas.as_bytes());

    // Finally, we encode the second-level entries.
    for (_, _, encoding_array) in second_level_entries {
        let record = CompressedPage {
            kind: UNWIND_SECOND_LEVEL_COMPRESSED,
            functions_offset: size_of::<CompressedPage>() as u16,
            functions_len: encoding_array.len() as u16,
            local_opcodes_offset: 0,
            local_opcodes_len: 0,
        };
        out.extend(record.as_bytes());
        for (offset, encoding_index) in encoding_array {
            ensure!(
                offset >> 24 == 0,
                "compressed page function offset does not fit into 24 bits: {offset}"
            );
            out.extend(&offset.as_bytes()[..3]);
            out.extend(encoding_index.as_bytes());
        }
    }

    Ok(out)
}
