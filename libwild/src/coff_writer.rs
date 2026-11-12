#![allow(clippy::unnecessary_wraps, clippy::unused_self)]

use crate::OutputFileData;
use crate::bail;
use crate::coff;
use crate::coff::Coff;
use crate::coff::CoffRelocation;
use crate::coff::DosHeader;
use crate::coff::output_section_id;
use crate::coff::part_id;
use crate::debug_assert_bail;
use crate::ensure;
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
use crate::layout::Resolution;
use crate::layout::Section;
use crate::layout::SyntheticSymbolsLayout;
use crate::output_section_id::OrderEvent;
use crate::output_section_id::SectionOutputInfo;
use crate::output_section_map::OutputSectionMap;
use crate::output_section_part_map::OutputSectionPartMap;
use crate::output_trace::HexU64;
use crate::output_trace::TraceOutput;
use crate::platform;
use crate::platform::Arch;
use crate::platform::Args as _;
use crate::platform::ObjectFile as _;
use crate::platform::Relocation;
use crate::platform::RelocationSequence as _;
use crate::platform::SectionAttributes;
use crate::platform::SectionHeader as _;
use crate::resolution::SectionSlot;
use crate::symbol_db::SymbolDb;
use crate::symbol_db::SymbolId;
use crate::timing_phase;
use crate::value_flags::PerSymbolFlags;
use crate::value_flags::ValueFlags;
use crate::verbose_timing_phase;
use linker_utils::elf::RelocationKind;
use linker_utils::elf::RelocationSize;
use linker_utils::utils::slice_from_all_bytes_mut;
use object::LittleEndian;
use object::from_bytes_mut;
use object::write::WritableBuffer;
use rayon::iter::IndexedParallelIterator as _;
use rayon::iter::IntoParallelIterator as _;
use rayon::iter::ParallelIterator as _;
use std::fmt::Display;
use std::sync::atomic::Ordering::Relaxed;
use tracing::debug_span;

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
    buffers: &mut OutputSectionPartMap<&mut [u8]>,
    _table_writer: &mut TableWriter,
    layout: &CoffLayout<'data>,
    trace: &TraceOutput,
) -> Result {
    verbose_timing_phase!("Write object", file_id = object.file_id.as_u32());

    let _span = debug_span!("write_file", filename = %object.input).entered();
    let _file_span = layout.args().common().trace_span_for_file(object.file_id);

    for (i, sec) in object.sections.iter().enumerate() {
        let section_index = object::SectionIndex(i);

        match sec {
            SectionSlot::Loaded(sec) => {
                write_object_section::<A>(object, layout, *sec, section_index, buffers, trace)?;
            }
            _ => (),
        }
    }

    Ok(())
}

fn write_object_section<'data, A: Arch<Platform = Coff>>(
    object: &ObjectLayout<'data, Coff>,
    layout: &CoffLayout<'data>,
    section: Section,
    section_index: object::SectionIndex,
    buffers: &mut OutputSectionPartMap<&mut [u8]>,
    trace: &TraceOutput,
) -> Result {
    let out = write_section_raw::<A>(object, layout, section, section_index, buffers)?;

    let relocations = object.relocations(section_index)?;
    apply_relocations::<A, CoffRelocation, _>(
        object,
        out,
        section_index,
        relocations.rel_iter(),
        layout,
        trace,
    )
    .with_context(|| {
        format!(
            "Failed to apply relocations in section `{}` of {}",
            object.object.section_display_name(section_index),
            object.input
        )
    })?;
    Ok(())
}

fn write_section_raw<'out, 'data, A: Arch<Platform = Coff>>(
    object: &ObjectLayout<'data, Coff>,
    layout: &CoffLayout,
    sec: Section,
    section_index: object::SectionIndex,
    buffers: &'out mut OutputSectionPartMap<&mut [u8]>,
) -> Result<&'out mut [u8]> {
    let part_id = object.section_part_id(section_index, &layout.symbol_db.section_part_ids);
    if layout
        .output_sections
        .has_data_in_file(part_id.output_section_id::<Coff>())
    {
        let section_buffer = buffers.get_mut(part_id);
        let allocation_size = sec.capacity(part_id, &layout.output_sections) as usize;
        if section_buffer.len() < allocation_size {
            bail!(
                "Insufficient space allocated to section `{}`. Tried to take {} bytes, but only {} remain",
                object.object.section_display_name(section_index),
                allocation_size,
                section_buffer.len()
            );
        }
        let out = section_buffer.split_off_mut(..allocation_size).unwrap();
        let object_section = object.object.section(section_index)?;
        if object_section.is_no_bits() {
            return Ok(out);
        }

        let section_info = layout
            .output_sections
            .output_info(part_id.output_section_id::<Coff>());
        let section_size = object.object.section_size(object_section)?;
        let section_size = section_size as usize;
        object
            .object
            .copy_section_data(object_section, &mut out[..section_size])?;
        fill_section_padding::<A>(&mut out[section_size..], section_info);

        Ok(&mut out[..section_size])
    } else {
        Ok(&mut [])
    }
}
fn apply_relocations<
    'data,
    A: Arch<Platform = Coff>,
    R: Relocation<Platform = Coff>,
    I: Iterator<Item = R>,
>(
    object: &ObjectLayout<'data, Coff>,
    out: &mut [u8],
    section_index: object::SectionIndex,
    relocations: I,
    layout: &CoffLayout<'data>,
    trace: &TraceOutput,
) -> Result {
    let section_address = object.section_resolutions[section_index.0]
        .address()
        .context("Attempted to apply relocations to a section that we didn't load")?;

    let mut relocation_count = 0;

    for rel in relocations {
        relocation_count += 1;

        let offset_in_section = rel.offset();

        apply_relocation::<A, R>(
            object,
            offset_in_section,
            &rel,
            section_address,
            layout,
            out,
            trace,
        )
        .with_context(|| {
            format!(
                "Failed to apply {} at offset 0x{offset_in_section:x}",
                display_relocation::<A, R>(object, &rel, layout)
            )
        })?;
    }

    layout
        .relocation_statistics
        .get(
            object
                .section_part_id(section_index, &layout.symbol_db.section_part_ids)
                .output_section_id::<Coff>(),
        )
        .fetch_add(relocation_count, Relaxed);
    Ok(())
}

fn display_relocation<'a, 'data, A: Arch<Platform = Coff>, R: Relocation>(
    object: &'a ObjectLayout<'data, Coff>,
    rel: &'a R,
    layout: &'a CoffLayout<'data>,
) -> DisplayRelocation<'a, 'data, A, R> {
    DisplayRelocation::<'a, 'data, A, R> {
        rel,
        symbol_db: &layout.symbol_db,
        per_symbol_flags: &layout.per_symbol_flags,
        object,
        phantom: std::marker::PhantomData,
    }
}

struct DisplayRelocation<'a, 'data, A: Arch<Platform = Coff>, R: Relocation> {
    rel: &'a R,
    symbol_db: &'a SymbolDb<'data, Coff>,
    per_symbol_flags: &'a PerSymbolFlags,
    object: &'a ObjectLayout<'data, Coff>,
    phantom: std::marker::PhantomData<A>,
}

impl<'a, 'data, A: Arch<Platform = Coff>, R: Relocation<Platform = Coff>> Display
    for DisplayRelocation<'a, 'data, A, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "relocation of type {} to ",
            A::rel_type_to_string(self.rel.raw_type())
        )?;
        match self.rel.symbol() {
            None => write!(f, "absolute")?,
            Some(local_symbol_index) => {
                let symbol_id = self.object.symbol_id_range.input_to_id(local_symbol_index);
                write!(
                    f,
                    "{}",
                    self.symbol_db
                        .symbol_debug(self.per_symbol_flags, symbol_id)
                )?;
            }
        }
        Ok(())
    }
}

struct RelocationResolution {
    resolution: Resolution<Coff>,
    local_symbol_id: SymbolId,
    flags: ValueFlags,
}

#[inline(always)]
fn get_resolution<'data, R: Relocation>(
    rel: &R,
    object_layout: &ObjectLayout<'data, Coff>,
    layout: &CoffLayout<'data>,
) -> Result<RelocationResolution> {
    let symbol_index = rel.symbol().context("Unsupported absolute relocation")?;
    let local_symbol_id = object_layout.symbol_id_range.input_to_id(symbol_index);
    let sym = object_layout.object.symbol(symbol_index)?;
    let section_index = object_layout.object.symbol_section(sym, symbol_index)?;
    let flags = layout.flags_for_symbol(local_symbol_id);
    let resolution = layout
        .symbol_resolution_with_flags(local_symbol_id, flags)
        .or_else(|| {
            section_index.and_then(|section_index| {
                let section_address =
                    object_layout.section_resolutions[section_index.0].address()?;
                let output_offset = platform::Symbol::value(sym);

                Some(Resolution {
                    raw_value: section_address + output_offset,
                    dynamic_symbol_index: None,
                    flags: ValueFlags::empty(),
                    format_specific: Default::default(),
                })
            })
        })
        .with_context(|| {
            format!(
                "Missing resolution for: {}",
                layout.symbol_debug(local_symbol_id)
            )
        })?;
    Ok(RelocationResolution {
        resolution,
        local_symbol_id,
        flags,
    })
}

#[inline(always)]
fn apply_relocation<'data, A: Arch<Platform = Coff>, R: Relocation<Platform = Coff>>(
    object_layout: &ObjectLayout<'data, Coff>,
    offset_in_section: u64,
    rel: &R,
    section_address: u64,
    layout: &CoffLayout<'data>,
    out: &mut [u8],
    trace: &TraceOutput,
) -> Result<()> {
    let place = section_address + offset_in_section;
    let _span = tracing::trace_span!(
        "relocation",
        address = place,
        address_hex = %HexU64::new(place)
    )
    .entered();

    let r_type = rel.raw_type();
    let rel_info = A::relocation_from_raw(rel.raw_type())?;
    ensure!(
        rel_info.implicit_addend,
        "PE/COFF relocations always have implicit addend"
    );
    let mut addend = rel.addend() as u64;
    // TODO: Copied from macho and is also similar to `adjust_value_based_on_content`. Perhaps
    // candidate for a deduplication?
    {
        let RelocationSize::ByteSize(byte_size @ (4 | 8)) = rel_info.size else {
            bail!(
                "Unexpected relocation size for implicit addend read: {}",
                rel_info.size
            );
        };
        let byte_size = usize::from(byte_size);
        let bytes = out
            .get(offset_in_section as usize..)
            .and_then(|bytes| bytes.get(..byte_size))
            .context("implicit addend read extends beyond section data")?;
        let mut buffer = [0u8; 8];
        buffer[..byte_size].copy_from_slice(bytes);
        let field = match byte_size {
            4 => i64::from(i32::from_le_bytes(bytes.try_into()?)) as u64,
            8 => u64::from_le_bytes(bytes.try_into()?),
            _ => todo!(),
        };
        addend = addend.wrapping_add(field);
    }

    match rel_info.kind {
        RelocationKind::None => return Ok(()),
        RelocationKind::Relative if rel.symbol().is_none() => {
            if layout.symbol_db.output_kind.is_position_independent() {
                bail!(
                    "relocation of type {} to absolute address cannot be used in \
                    position-independent output; recompile with -fPIC",
                    rel.raw_type()
                );
            }
            let place = section_address + offset_in_section;
            let value = (rel.addend() as u64).wrapping_sub(place);
            rel_info.write_to_buffer(value, &mut out[offset_in_section as usize..])?;
            return Ok(());
        }
        RelocationKind::Absolute
            if layout.symbol_db.output_kind.is_shared_object()
                && A::is_disallowed_in_shared_object(r_type) =>
        {
            bail!(
                "relocation type {} cannot be used when making a shared object; \
                 recompile with -fPIC",
                A::rel_type_to_string(r_type)
            );
        }
        _ => {}
    }
    let RelocationResolution {
        resolution,
        local_symbol_id,
        flags,
    } = get_resolution(rel, object_layout, layout)?;
    let reject_dynamic_direct = layout.symbol_db.output_kind.is_position_independent()
        || A::relocation_from_raw(r_type)?.kind == RelocationKind::TpOff;
    if reject_dynamic_direct
        && (flags.is_interposable() || flags.is_dynamic())
        && !flags.needs_copy_relocation()
        && !flags.needs_plt()
        && A::is_disallowed_for_interposable_symbols(r_type)
    {
        bail!(
            "relocation {} cannot be used against symbol; recompile with -fPIC",
            A::rel_type_to_string(r_type)
        );
    }
    let rel_info = A::relocation_from_raw(r_type)?;

    let value = match rel_info.kind {
        // TODO: This reuses ELF code which doesn't fit well here. Create COFF's own code in linker
        // utils.
        RelocationKind::Absolute if r_type == object::pe::IMAGE_REL_AMD64_ADDR32NB => {
            let without_base = addend.wrapping_sub(layout.args().image_base().unwrap());
            resolution.value_with_addend(without_base as i64)
        }
        RelocationKind::Absolute => resolution.value_with_addend(addend as i64),
        RelocationKind::Relative => {
            // TODO: Handle undefined weak
            let symbol_plus_addend = resolution.value_with_addend(addend as i64);
            symbol_plus_addend.wrapping_sub(place)
        }
        RelocationKind::None => 0,
        _ => {
            unreachable!()
        }
    };

    let offset_in_section = offset_in_section as usize;

    trace.emit(place, || {
        format!(
            "relocation applied flags={flags},\n\
                rel_kind={rel_kind:?},\n\
                value=0x{value:x}, symbol_name={symbol_name}",
            rel_kind = rel_info.kind,
            symbol_name = layout.symbol_db.symbol_name_for_display(local_symbol_id),
        )
    });
    tracing::trace!(
            %flags,
            ?rel_info.kind,
            %rel_info.size,
            value,
            value_hex = %HexU64::new(value),
            symbol_name = %layout.symbol_db.symbol_name_for_display(local_symbol_id),
            "relocation applied");

    rel_info.write_to_buffer(value, &mut out[offset_in_section..])?;

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

fn write_synthetic_symbols<'data, A: Arch<Platform = Coff>>(
    _syn: &SyntheticSymbolsLayout<Coff>,
    _table_writer: &mut TableWriter<'_, '_>,
    _layout: &CoffLayout<'data>,
) -> Result {
    verbose_timing_phase!("Write synthetic symbols");

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
