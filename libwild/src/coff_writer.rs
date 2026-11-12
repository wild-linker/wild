#![allow(clippy::unnecessary_wraps, clippy::unused_self)]

use crate::OutputFileData;
use crate::bail;
use crate::coff;
use crate::coff::Coff;
use crate::coff::DosHeader;
use crate::coff::output_section_id;
use crate::coff::part_id;
use crate::debug_assert_bail;
use crate::error::Context as _;
use crate::error::Result;
use crate::file_writer::SizedOutput;
use crate::file_writer::excessive_allocation;
use crate::file_writer::insufficient_allocation;
use crate::file_writer::split_buffers_by_alignment;
use crate::file_writer::split_output_by_group;
use crate::file_writer::split_output_into_sections;
use crate::layout::EpilogueLayout;
use crate::layout::FileLayout;
use crate::layout::HeaderInfo;
use crate::layout::Layout;
use crate::layout::ObjectLayout;
use crate::layout::PreludeLayout;
use crate::layout::SyntheticSymbolsLayout;
use crate::output_section_id::OrderEvent;
use crate::output_section_id::SectionOutputInfo;
use crate::output_section_map::OutputSectionMap;
use crate::output_section_part_map::OutputSectionPartMap;
use crate::output_trace::TraceOutput;
use crate::platform::Arch;
use crate::platform::Args as _;
use crate::platform::SectionAttributes;
use crate::timing_phase;
use crate::verbose_timing_phase;
use linker_utils::utils::slice_from_all_bytes_mut;
use object::LittleEndian;
use object::from_bytes_mut;
use object::write::WritableBuffer;
use rayon::iter::IndexedParallelIterator as _;
use rayon::iter::IntoParallelIterator as _;
use rayon::iter::ParallelIterator as _;
use std::sync::atomic::Ordering::Relaxed;

// TODO: audit unwraps in this file. They are much more helpful for the development phase compared
// to bare `?`, but we want to get rid of user facing ones before `coff` feature is enabled by
// default.

type CoffLayout<'data> = Layout<'data, Coff>;

pub(crate) fn write<'data, A: Arch<Platform = Coff>>(
    sized_output: &mut SizedOutput<impl OutputFileData>,
    layout: &CoffLayout<'data>,
) -> Result {
    write_file_contents::<A>(sized_output, layout)?;
    // TODO: elf specific right now
    // if layout.args().common().validate_output {
    //     crate::validation::validate_bytes(layout, &sized_output.out)?;
    // }

    Ok(())
}

fn write_file_contents<'data, A: Arch<Platform = Coff>>(
    sized_output: &mut SizedOutput<impl OutputFileData>,
    layout: &CoffLayout<'data>,
) -> Result {
    timing_phase!("Write data to file");
    let (mut section_buffers, padding) = split_output_into_sections(layout, &mut sized_output.out);

    fill_padding_for_sections::<A>(layout, padding);

    let mut writable_buckets = split_buffers_by_alignment(&mut section_buffers, layout);
    let groups_and_buffers = split_output_by_group(layout, &mut writable_buckets);
    groups_and_buffers
        .into_par_iter()
        .with_max_len(1)
        .try_for_each(|(group, mut buffers)| -> Result {
            verbose_timing_phase!("Write group");

            let mut table_writer = TableWriter::from_layout(layout, &mut buffers);

            for file in &group.files {
                write_file::<A>(
                    file,
                    &mut buffers,
                    &mut table_writer,
                    layout,
                    &sized_output.trace,
                )
                .with_context(|| format!("Failed copying from {file} to output file"))?;
            }
            table_writer
                .validate_empty(&group.mem_sizes)
                .with_context(|| format!("validate_empty failed for {group}"))?;
            Ok(())
        })?;

    for (output_section_id, _) in layout.output_sections.ids_with_info() {
        let relocations = layout
            .relocation_statistics
            .get(output_section_id)
            .load(Relaxed);

        if relocations > 0 {
            tracing::debug!(
                target: "metrics",
                section = layout.output_sections.display_name(output_section_id),
                relocations, "resolved relocations");
        }
    }

    fill_padding(section_buffers);

    Ok(())
}

fn fill_padding_for_sections<A: Arch<Platform = Coff>>(
    layout: &Layout<'_, Coff>,
    padding: crate::file_writer::PaddingSlices<'_>,
) {
    timing_phase!("Fill padding for sections");

    for pslice in padding.slices {
        if let Some(section_id) = pslice.parent_section_id {
            let section_info = layout.output_sections.output_info(section_id);
            fill_section_padding::<A>(pslice.slice, section_info);
        } else {
            pslice.slice.fill(0);
        }
    }
}

fn fill_padding(mut section_buffers: OutputSectionMap<&mut [u8]>) {
    section_buffers.for_each_mut(|_, out| {
        out.fill(0);
    });
}

fn write_file<'data, A: Arch<Platform = Coff>>(
    file: &FileLayout<'data, Coff>,
    buffers: &mut OutputSectionPartMap<&mut [u8]>,
    table_writer: &mut TableWriter<'_, '_>,
    layout: &CoffLayout<'data>,
    trace: &TraceOutput,
) -> Result {
    match file {
        FileLayout::Object(s) => {
            write_object::<A>(s, buffers, table_writer, layout, trace)?;
        }
        FileLayout::Prelude(s) => {
            write_prelude::<A>(s, buffers, table_writer, layout)?;
        }
        FileLayout::Epilogue(s) => write_epilogue::<A>(s, buffers, table_writer, layout, trace)?,
        FileLayout::SyntheticSymbols(s) => {
            write_synthetic_symbols::<A>(s, table_writer, layout)?;
        }
        FileLayout::LinkerScript(_s) => bail!("Wild's PE/COFF doesn't support linker scripts yet"),
        FileLayout::NotLoaded | FileLayout::StubLibrary(_) => {}
        FileLayout::Dynamic(_s) => {
            // write_dynamic_file::<A>(s, table_writer, layout)?
        }
    }

    Ok(())
}

struct TableWriter<'layout, 'out> {
    section_headers: &'out mut [coff::SectionHeader],
    _phantom: std::marker::PhantomData<&'layout ()>,
}

impl<'layout, 'out> TableWriter<'layout, 'out> {
    fn from_layout(
        _layout: &'layout CoffLayout,
        buffers: &mut OutputSectionPartMap<&'out mut [u8]>,
    ) -> TableWriter<'layout, 'out> {
        Self::new(buffers)
    }

    fn new(buffers: &mut OutputSectionPartMap<&'out mut [u8]>) -> TableWriter<'layout, 'out> {
        TableWriter {
            section_headers: slice_from_all_bytes_mut(buffers.take(part_id::SECTION_HEADERS)),
            _phantom: std::marker::PhantomData,
        }
    }

    fn take_section_header(&mut self) -> Result<&'out mut coff::SectionHeader> {
        self.section_headers
            .split_off_first_mut()
            .ok_or_else(|| insufficient_allocation("section headers"))
    }

    /// Checks that we used all of the entries that we requested during layout.
    fn validate_empty(&self, mem_sizes: &OutputSectionPartMap<u64>) -> Result {
        if !self.section_headers.is_empty() {
            return Err(excessive_allocation(
                "section headers",
                size_of_val(self.section_headers) as u64,
                mem_sizes.get(part_id::SECTION_HEADERS),
            ));
        }

        Ok(())
    }
}

fn write_object<'data, A: Arch<Platform = Coff>>(
    object: &ObjectLayout<'data, Coff>,
    _buffers: &mut OutputSectionPartMap<&mut [u8]>,
    _table_writer: &mut TableWriter,
    _layout: &CoffLayout<'data>,
    _trace: &TraceOutput,
) -> Result {
    verbose_timing_phase!("Write object", file_id = object.file_id.as_u32());

    Ok(())
}

fn write_synthetic_symbols<'data, A: Arch<Platform = Coff>>(
    _syn: &SyntheticSymbolsLayout<Coff>,
    _table_writer: &mut TableWriter<'_, '_>,
    _layout: &CoffLayout<'data>,
) -> Result {
    verbose_timing_phase!("Write synthetic symbols");

    Ok(())
}

fn write_prelude<A: Arch<Platform = Coff>>(
    prelude: &PreludeLayout<Coff>,
    buffers: &mut OutputSectionPartMap<&mut [u8]>,
    table_writer: &mut TableWriter<'_, '_>,
    layout: &CoffLayout,
) -> Result {
    verbose_timing_phase!("Write prelude");

    let dos_header: &mut DosHeader = from_bytes_mut(buffers.get_mut(crate::part_id::FILE_HEADER))
        .map_err(|_| insufficient_allocation("DOS header"))?
        .0;
    populate_dos_header::<A>(layout, dos_header);

    let dos_stub: &mut [u8; coff::DOS_STUB.len()] =
        from_bytes_mut(buffers.get_mut(part_id::DOS_STUB))
            .map_err(|_| insufficient_allocation("DOS stub"))?
            .0;
    write_dos_stub::<A>(dos_stub);

    let (nt_header, data_directory_bytes): (&mut coff::NtHeader, _) =
        from_bytes_mut(buffers.get_mut(part_id::NT_HEADER))
            .map_err(|_| insufficient_allocation("NT header"))?;
    populate_nt_header::<A>(layout, &prelude.header_info, nt_header)?;
    // TODO: `object` doesn't include this in `coff::NtHeader`, so just wipe remaining bytes for
    // now.
    data_directory_bytes.fill(0);

    write_section_headers(table_writer, layout)?;

    Ok(())
}

fn populate_dos_header<A: Arch<Platform = Coff>>(layout: &CoffLayout, header: &mut DosHeader) {
    let e = LittleEndian;
    header.e_magic.set(e, object::pe::IMAGE_DOS_SIGNATURE);
    header.e_cblp.set(e, Default::default());
    header.e_cp.set(e, Default::default());
    header.e_crlc.set(e, Default::default());
    header.e_cparhdr.set(
        e,
        u16::try_from(size_of::<DosHeader>() / u16::BITS as usize).unwrap(),
    );
    header.e_minalloc.set(e, Default::default());
    header.e_maxalloc.set(e, Default::default());
    header.e_ss.set(e, Default::default());
    header.e_sp.set(e, Default::default());
    header.e_csum.set(e, Default::default());
    header.e_ip.set(e, Default::default());
    header.e_cs.set(e, Default::default());
    header.e_lfarlc.set(e, Default::default());
    header.e_ovno.set(e, Default::default());
    header.e_res = [0.into(); 4];
    header.e_oemid.set(e, Default::default());
    header.e_oeminfo.set(e, Default::default());
    header.e_res2 = [0.into(); 10];
    header.e_lfanew.set(
        e,
        u32::try_from(
            layout
                .section_layouts
                .get(output_section_id::NT_HEADER)
                .file_offset,
        )
        .unwrap(),
    );
}

fn write_dos_stub<A: Arch<Platform = Coff>>(mut bytes: &mut [u8]) {
    bytes.write_bytes(coff::DOS_STUB);
}

fn populate_nt_header<A: Arch<Platform = Coff>>(
    layout: &CoffLayout,
    header_info: &HeaderInfo,
    header: &mut coff::NtHeader,
) -> Result {
    let output_kind = layout.symbol_db.output_kind;
    let mut characteristics =
        object::pe::IMAGE_FILE_EXECUTABLE_IMAGE | object::pe::IMAGE_FILE_LARGE_ADDRESS_AWARE;
    if output_kind.is_shared_object() {
        characteristics |= object::pe::IMAGE_FILE_DLL;
    }
    let e = LittleEndian;

    header.signature.set(e, object::pe::IMAGE_NT_SIGNATURE);

    // File header
    header
        .file_header
        .machine
        .set(e, object::pe::IMAGE_FILE_MACHINE_AMD64);
    header.file_header.number_of_sections.set(
        e,
        u16::try_from(header_info.num_output_sections_with_content).unwrap(),
    );
    header
        .file_header
        .time_date_stamp
        .set(e, Default::default());
    header.file_header.pointer_to_symbol_table.set(e, 0);
    header.file_header.number_of_symbols.set(e, 0);
    header
        .file_header
        .size_of_optional_header
        .set(e, coff::SIZE_OF_OPTIONAL_HEADER64 as u16);
    header.file_header.characteristics.set(e, characteristics);

    // Optional header
    header
        .optional_header
        .magic
        .set(e, object::pe::IMAGE_NT_OPTIONAL_HDR64_MAGIC);
    header.optional_header.major_linker_version = 14;
    header.optional_header.minor_linker_version = 0;
    header.optional_header.size_of_code.set(e, 0);
    // TODO: Add `is_prog_bits` helper
    header.optional_header.size_of_initialized_data.set(
        e,
        u32::try_from(
            layout
                .output_sections
                .ids_with_info()
                .filter(|(_, sec)| {
                    sec.section_attributes
                        .flags
                        .contains(object::pe::IMAGE_SCN_CNT_INITIALIZED_DATA)
                })
                .map(|(id, _)| layout.section_layouts.get(id).file_size)
                .sum::<usize>(),
        )
        .unwrap(),
    );
    header.optional_header.size_of_uninitialized_data.set(
        e,
        u32::try_from(
            layout
                .output_sections
                .ids_with_info()
                .filter(|(_, sec)| sec.section_attributes.is_no_bits())
                .map(|(id, _)| layout.section_layouts.get(id).file_size)
                .sum::<usize>(),
        )
        .unwrap(),
    );
    header.optional_header.address_of_entry_point.set(
        e,
        // TODO: Skips a lot of relevant logic compared to ELF counterpart
        va_to_rva(layout.resolved_entry_symbol_address()?.unwrap(), layout),
    );
    header.optional_header.base_of_code.set(e, 0);
    header
        .optional_header
        .image_base
        .set(e, layout.args().image_base().unwrap());
    header.optional_header.section_alignment.set(
        e,
        u32::try_from(layout.args().loadable_segment_alignment().value()).unwrap(),
    );
    // TODO: Wild doesn't support separate memory and file alignment
    header.optional_header.file_alignment.set(
        e,
        u32::try_from(layout.args().loadable_segment_alignment().value()).unwrap(),
    );
    header
        .optional_header
        .major_operating_system_version
        .set(e, 6);
    header
        .optional_header
        .minor_operating_system_version
        .set(e, Default::default());
    header
        .optional_header
        .major_image_version
        .set(e, Default::default());
    header
        .optional_header
        .minor_image_version
        .set(e, Default::default());
    header.optional_header.major_subsystem_version.set(e, 6);
    header
        .optional_header
        .minor_subsystem_version
        .set(e, Default::default());
    header
        .optional_header
        .win32_version_value
        .set(e, Default::default());
    header.optional_header.size_of_image.set(
        e,
        va_to_rva(
            layout
                .section_layouts
                .iter()
                .map(|(_, sec)| sec.mem_end())
                .max()
                .unwrap(),
            layout,
        ),
    );
    header.optional_header.size_of_headers.set(
        e,
        u32::try_from(
            // TODO: this is wrong, doesn't match alignment requirements
            layout
                .section_layouts
                .get(output_section_id::DOS_STUB)
                .file_size
                + layout
                    .section_layouts
                    .get(output_section_id::NT_HEADER)
                    .file_size,
        )
        .unwrap(),
    );
    header.optional_header.check_sum.set(e, Default::default());
    header
        .optional_header
        .subsystem
        .set(e, object::pe::IMAGE_SUBSYSTEM_WINDOWS_CUI);
    header
        .optional_header
        .dll_characteristics
        .set(e, Default::default());
    header
        .optional_header
        .size_of_stack_reserve
        .set(e, Default::default());
    header
        .optional_header
        .size_of_stack_commit
        .set(e, Default::default());
    header
        .optional_header
        .size_of_heap_reserve
        .set(e, Default::default());
    header
        .optional_header
        .size_of_heap_commit
        .set(e, Default::default());
    header
        .optional_header
        .loader_flags
        .set(e, Default::default());
    header.optional_header.number_of_rva_and_sizes.set(
        e,
        u32::try_from(object::pe::IMAGE_NUMBEROF_DIRECTORY_ENTRIES).unwrap(),
    );

    Ok(())
}

fn write_epilogue<A: Arch<Platform = Coff>>(
    _epilogue: &EpilogueLayout<Coff>,
    _buffers: &mut OutputSectionPartMap<&mut [u8]>,
    _table_writer: &mut TableWriter<'_, '_>,
    _layout: &CoffLayout<'_>,
    _trace: &TraceOutput,
) -> Result {
    verbose_timing_phase!("Write epilogue");

    Ok(())
}

fn write_section_headers(table_writer: &mut TableWriter<'_, '_>, layout: &CoffLayout) -> Result {
    let output_sections = &layout.output_sections;

    let mut order = layout.output_order.into_iter().peekable();

    while let Some(event) = order.next() {
        let OrderEvent::Section(section_id) = event else {
            continue;
        };

        let section_layout = layout.merged_section_layouts.get(section_id);

        if output_sections
            .output_index_of_section(section_id)
            .is_none()
        {
            continue;
        }

        while let Some(OrderEvent::Section(next_section_id)) = order.peek()
            && let Some(primary_id) = output_sections.merge_target(*next_section_id)
        {
            debug_assert_bail!(
                primary_id == section_id,
                "Section order mismatch {} != {}",
                output_sections.section_debug(primary_id),
                output_sections.section_debug(section_id),
            );
            order.next();
        }

        let name = layout.output_sections.name(section_id).with_context(|| {
            format!(
                "Missing name for section {}",
                layout.output_sections.section_debug(section_id)
            )
        })?;
        let mut name_bytes = [0u8; 8];
        if name.len() > 8
            && layout
                .output_sections
                .section_flags(section_id)
                .contains(object::pe::IMAGE_SCN_MEM_DISCARDABLE)
        {
            eprintln!(
                "Long section names are not supported yet, section name for `{name}` will be truncated"
            );
        }
        let byte_count = name.len().min(name_bytes.len());
        name_bytes[..byte_count].copy_from_slice(&name.bytes()[..byte_count]);

        let entry = table_writer.take_section_header()?;
        entry.name = name_bytes;
        entry
            .virtual_size
            .set(LittleEndian, section_layout.mem_size.try_into()?);
        entry
            .virtual_address
            .set(LittleEndian, va_to_rva(section_layout.mem_offset, layout));
        let (size_of_raw_data, pointer_to_raw_data) =
            if output_sections.has_data_in_file(section_id) {
                (
                    section_layout
                        .alignment
                        .align_up(section_layout.file_size as u64)
                        .try_into()?,
                    section_layout.file_offset.try_into()?,
                )
            } else {
                (0, 0)
            };
        entry.size_of_raw_data.set(LittleEndian, size_of_raw_data);
        entry
            .pointer_to_raw_data
            .set(LittleEndian, pointer_to_raw_data);
        entry
            .pointer_to_relocations
            .set(LittleEndian, Default::default());
        entry
            .pointer_to_linenumbers
            .set(LittleEndian, Default::default());
        entry
            .number_of_relocations
            .set(LittleEndian, Default::default());
        entry
            .number_of_linenumbers
            .set(LittleEndian, Default::default());
        entry.characteristics.set(
            LittleEndian,
            layout
                .output_sections
                .section_flags(section_id)
                .without(object::pe::SectionFlags(object::pe::IMAGE_SCN_ALIGN_MASK)),
        );
    }
    Ok(())
}

fn fill_section_padding<A: Arch<Platform = Coff>>(
    padding: &mut [u8],
    section_info: &SectionOutputInfo<Coff>,
) {
    if let Some(pattern) = section_info.fill {
        let chunks = padding.chunks_mut(4);
        for chunk in chunks {
            let len = chunk.len();
            chunk.copy_from_slice(&pattern[..len]);
        }
    } else {
        // TODO: fill_section_padding uses ELF flags
        // A::fill_section_padding(padding, section_info.section_attributes.flags);
        padding.fill(0);
    }
}

/// Virtual Address to Relative Virtual Address
fn va_to_rva(address: u64, layout: &CoffLayout) -> u32 {
    u32::try_from(if let Some(base) = layout.args().image_base() {
        address - base
    } else {
        address
    })
    .unwrap()
}
