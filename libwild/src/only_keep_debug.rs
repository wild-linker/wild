// Post-layout pass for --only-keep-debug.
// Zeroes file_size for alloc non-NOTE sections, then recalculates file offsets.

use crate::compression::recalculate_file_offsets;
use crate::elf;
use crate::elf::ElfClass;
use crate::layout::Layout;
use crate::platform::SectionFlags as _;
use crate::timing_phase;
use linker_utils::elf::SectionType;
use linker_utils::elf::sht;

/// Returns whether this section should be hollowed (file_size zeroed, sh_type set to SHT_NOBITS)
/// under `--only-keep-debug`. Allocatable sections are hollowed except for SHT_NULL and SHT_NOTE.
///
/// This predicate is shared between `zero_alloc_section_sizes` (post-layout file size zeroing),
/// section content writing, and section header emission to keep them in sync.
pub(crate) fn should_hollow_section(is_alloc: bool, section_type: SectionType) -> bool {
    is_alloc && section_type != sht::NULL && section_type != sht::NOTE
}

pub(crate) fn maybe_only_keep_debug_elf<C: ElfClass>(layout: &mut Layout<elf::Elf<C>>) {
    if !layout.args().only_keep_debug() {
        return;
    }
    timing_phase!("Only-keep-debug: zero alloc sections");
    zero_alloc_section_sizes(layout);
    recalculate_file_offsets(layout);
}

fn zero_alloc_section_sizes<C: ElfClass>(layout: &mut Layout<elf::Elf<C>>) {
    // Don't zero linker-generated headers (file header, program headers, section headers).
    let header_ids = [
        crate::output_section_id::FILE_HEADER,
        elf::output_section_id::PROGRAM_HEADERS,
        elf::output_section_id::SECTION_HEADERS,
    ];
    for (section_id, _) in layout.output_sections.ids_with_info() {
        if header_ids.contains(&section_id) {
            continue;
        }
        let primary_id = layout.output_sections.primary_output_section(section_id);
        let flags = layout.output_sections.section_flags(primary_id);
        let section_type = layout
            .output_sections
            .output_info(primary_id)
            .section_attributes
            .ty;

        if !should_hollow_section(flags.is_alloc(), section_type) {
            continue;
        }

        for part_id in section_id.parts::<elf::Elf<C>>() {
            layout.section_part_layouts.get_mut(part_id).file_size = 0;
        }

        for group in &mut layout.group_layouts {
            for part_id in section_id.parts::<elf::Elf<C>>() {
                *group.file_sizes.get_mut(part_id) = 0;
            }
        }

        layout.section_layouts.get_mut(section_id).file_size = 0;
        layout.merged_section_layouts.get_mut(primary_id).file_size = 0;
    }
}
