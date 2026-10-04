//! Builds a Mach-O compact unwind section.

use crate::ensure;
use crate::error::Result;
use crate::macho::ResolvedUnwindInfo;
use crate::macho::UnwindInfoWithRelocs;
use crate::symbol_db::SymbolId;
use crate::verbose_timing_phase;
use anyhow::Context;
use hashbrown::HashMap;
use indexmap::IndexMap;
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

// For vast majority of the programs, we should be able to fit all opcodes (encodings)
// in the global palette, so that's why the local palette is rather small.
const PALETTE_MAXIMUM_INDEX: usize = u8::MAX as usize + 1;
const LOCAL_PALETTE_SIZE: usize = 32;
const GLOBAL_PALETTE_SIZE: usize = PALETTE_MAXIMUM_INDEX - LOCAL_PALETTE_SIZE;
const MAX_FUNCTION_OFFSET: u64 = 1 << 24;

#[derive(Default)]
struct EncodingPalette {
    global: IndexMap<u32, usize>,
    local: HashMap<u32, usize>,
}

impl EncodingPalette {
    fn record_encodings(&mut self, encodings: &[u32]) {
        let mut encoding_histogram = HashMap::new();
        for &encoding in encodings {
            *encoding_histogram.entry(encoding).or_insert(0usize) += 1;
        }
        let encoding_frequency = encoding_histogram
            .into_iter()
            .map(|(encoding, count)| (count, encoding))
            .sorted()
            .rev()
            .collect_vec();
        let (global_encodings, local_encodings) =
            encoding_frequency.split_at(encoding_frequency.len().min(GLOBAL_PALETTE_SIZE));
        self.global = global_encodings
            .iter()
            .map(|&(count, encoding)| (encoding, count))
            .collect();
        self.local = local_encodings
            .iter()
            .map(|&(count, encoding)| (encoding, count))
            .collect();
    }

    // The number of unique encodings indexable from the global palette.
    fn global_unique_encodings(&self) -> usize {
        self.global.len()
    }

    // The total number of encoding values indexed by global palette.
    fn total_global_encodings(&self) -> usize {
        self.global.values().sum()
    }

    // The total number of encoding values indexed by a local per-page palette.
    fn total_local_encodings(&self) -> usize {
        self.local.values().sum()
    }
}

#[derive(Default)]
struct CompressPageInProgress {
    local_palette: IndexMap<u32, ()>,
    // A pair of function start and the encoding index.
    functions: Vec<u32>,
    start: u64,
    lsda_count: usize,
}

struct FinishedCompressedPage {
    encoded: Vec<u8>,
    start: u64,
    lsda_count: usize,
}

enum PageBuilder {
    InProgress(CompressPageInProgress),
    Finished(FinishedCompressedPage),
}

impl PageBuilder {
    fn new(start: u64) -> Self {
        Self::InProgress(CompressPageInProgress {
            start,
            ..Default::default()
        })
    }

    /// Closes the page if the entry does not fit. In that case the caller must
    /// retry the entry on a fresh page; it is not included in the finished page.
    fn encode(
        self,
        address: u64,
        encoding: u32,
        has_lsda: bool,
        palette: &EncodingPalette,
    ) -> Self {
        let Self::InProgress(mut page) = self else {
            unreachable!("InProgress expected");
        };
        let offset = address - page.start;
        let global_count = palette.global_unique_encodings();
        let global_index = palette.global.get_index_of(&encoding);
        let local_index = page.local_palette.get_index_of(&encoding);
        let needs_local = global_index.is_none() && local_index.is_none();
        let local_count = page.local_palette.len() + if needs_local { 1 } else { 0 };
        if offset >= MAX_FUNCTION_OFFSET
            // Local palette is full.
            || global_count + local_count > PALETTE_MAXIMUM_INDEX
            // The page is full at this point.
            || page.functions.len() + 1 + local_count > COMPRESSED_PAGE_ENTRIES_COUNT
        {
            return Self::Finished(page.finish());
        }

        let index = global_index.unwrap_or_else(|| {
            let index = local_index.unwrap_or_else(|| {
                let index = page.local_palette.len();
                page.local_palette.insert(encoding, ());
                index
            });
            global_count + index
        });
        debug_assert!(u8::try_from(index).is_ok());
        page.functions.push(((index as u32) << 24) | offset as u32);
        if has_lsda {
            page.lsda_count += 1;
        }
        Self::InProgress(page)
    }

    fn finish(self) -> FinishedCompressedPage {
        match self {
            Self::InProgress(page) => page.finish(),
            Self::Finished(page) => page,
        }
    }
}

impl CompressPageInProgress {
    fn finish(self) -> FinishedCompressedPage {
        let functions_size = self.functions.len() * size_of::<u32>();
        let header = CompressedPage {
            kind: UNWIND_SECOND_LEVEL_COMPRESSED,
            functions_offset: size_of::<CompressedPage>() as u16,
            functions_len: self.functions.len() as u16,
            local_opcodes_offset: (size_of::<CompressedPage>() + functions_size) as u16,
            local_opcodes_len: self.local_palette.len() as u16,
        };
        let mut out = Vec::with_capacity(COMPRESSED_PAGE_SIZE);
        out.extend(header.as_bytes());
        out.extend(self.functions.as_bytes());
        for encoding in self.local_palette.keys() {
            out.extend(encoding.as_bytes());
        }
        FinishedCompressedPage {
            encoded: out,
            start: self.start,
            lsda_count: self.lsda_count,
        }
    }
}

pub(crate) fn output_size(unwind_info_entries: &[UnwindInfoWithRelocs]) -> Result<u64> {
    verbose_timing_phase!("Estimate compact unwind section size");

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
        .collect_vec();
    let mut palette = EncodingPalette::default();
    palette.record_encodings(&encoding_values);

    // The worst case scenario would be use having a sequence of encodings that all need a local
    // palette and so we'll reach the page capacity because of the full palette. The estimation
    // is very gross, but as already mentioned, it's covering very unlikely situation.
    let locally_required_pages = palette.total_local_encodings().div_ceil(LOCAL_PALETTE_SIZE);
    let global_palette_count = palette.global_unique_encodings();
    let global_encodings = palette.total_global_encodings();
    let globally_required_pages = global_encodings.div_ceil(COMPRESSED_PAGE_ENTRIES_COUNT);

    let lsdas = unwind_info_entries
        .iter()
        .filter_map(|entry| entry.lsda_relocation)
        .count();

    // How much space will we need for encodings in a global palette.
    let mut compressed_pages_total_size =
        globally_required_pages * size_of::<CompressedPage>() + global_encodings * size_of::<u32>();
    // The upper limit assumes each entry will basically take 2 x u32 (one for palette, second for
    // index in the palette) and we'll have at most locally_required_pages such pages.
    compressed_pages_total_size += locally_required_pages
        * (LOCAL_PALETTE_SIZE * 2 * size_of::<u32>() + size_of::<CompressedPage>());

    // Plus, the offset can track only 24 bits (16MiB). For being sure, let's track
    // all functions larger than 4MiB and allocate extra pages for them.
    // TODO: Actually, there might be a situation where a series of medium-sized functions
    // will not fit into a single page. Investigate later if the situation arises.
    let large_functions = unwind_info_entries
        .iter()
        .filter(|entry| entry.entry.length >= (MAX_FUNCTION_OFFSET / 4) as u32)
        .count();
    compressed_pages_total_size += large_functions * size_of::<CompressedPage>();

    // PageEntry:CompressedPage mapping is 1:1 (modulo we need one more for the termination page)
    let page_entries_total_size =
        (locally_required_pages + globally_required_pages + large_functions + 1)
            * size_of::<PageEntry>();
    let root_total_size = size_of::<CompactUnwindInfoHeader>()
        + global_palette_count * size_of::<u32>()
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
    verbose_timing_phase!("Encode compact unwind section");

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
        .collect_vec();
    let mut palette = EncodingPalette::default();
    palette.record_encodings(&encoding_values);
    let global_encodings = palette.global.keys().copied().collect_vec();

    // Build variable-sized pages, retrying an entry on a fresh page when it
    // exceeds the address range, palette capacity, or byte capacity.
    let mut second_level_entries = Vec::new();
    let mut page = PageBuilder::new(unwind_info_entries[0].start_address);
    for (entry, encoding) in unwind_info_entries.iter().zip(encoding_values) {
        page = match page.encode(
            entry.start_address,
            encoding,
            entry.lsda_address.is_some(),
            &palette,
        ) {
            PageBuilder::Finished(finished) => {
                second_level_entries.push(finished);
                let next = PageBuilder::new(entry.start_address).encode(
                    entry.start_address,
                    encoding,
                    entry.lsda_address.is_some(),
                    &palette,
                );
                ensure!(
                    matches!(next, PageBuilder::InProgress(_)),
                    "compact unwind entry cannot fit into an empty compressed page"
                );
                next
            }
            in_progress @ PageBuilder::InProgress(_) => in_progress,
        };
    }
    second_level_entries.push(page.finish());
    let compressed_pages = second_level_entries.len();

    let mut out = Vec::with_capacity(section_size);

    let header_size = size_of::<CompactUnwindInfoHeader>();
    let encoding_size = global_encodings.as_bytes().len();
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
        global_opcodes_len: global_encodings.len() as u32,
        pages_offset: (header_size + encoding_size + personalities_size) as u32,
        pages_len: (compressed_pages + 1) as u32,
        personalities_offset: (header_size + encoding_size) as u32,
        personalities_len: personalities.len() as u32,
    };
    out.extend(header.as_bytes());
    out.extend(global_encodings.as_bytes());
    out.extend(personalities.as_bytes());
    let header_size = out.len();

    // Now having built second-level entries, we can build and encode the first-level entries.
    let lsda_data_offset = header_size + (compressed_pages + 1) * size_of::<PageEntry>();
    let second_level_first_offset = lsda_data_offset + lsdas.len() * size_of::<LsdaEntry>();

    let mut lsda_offset = 0;
    let mut page_offset = second_level_first_offset;
    for page in &second_level_entries {
        let record = PageEntry {
            first_address: u32::try_from(page.start - text_segment_start)
                .context("first address does not fit into u32")?,
            lsda_index_offset: (lsda_data_offset + lsda_offset * size_of::<LsdaEntry>()) as u32,
            page_offset: u32::try_from(page_offset).context("page offset does not fit into u32")?,
        };
        out.extend(record.as_bytes());
        lsda_offset += page.lsda_count;
        page_offset += page.encoded.len();
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
    for page in second_level_entries {
        out.extend(page.encoded);
    }

    Ok(out)
}
