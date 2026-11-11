#![allow(clippy::unnecessary_wraps, clippy::unused_self)]

use crate::OutputFileData;
use crate::bail;
use crate::coff;
use crate::coff::Coff;
use crate::coff::DosHeader;
use crate::coff::part_id;
use crate::error::Context as _;
use crate::error::Result;
use crate::file_writer::SizedOutput;
use crate::file_writer::insufficient_allocation;
use crate::file_writer::split_buffers_by_alignment;
use crate::file_writer::split_output_by_group;
use crate::file_writer::split_output_into_sections;
use crate::layout::EpilogueLayout;
use crate::layout::FileLayout;
use crate::layout::Layout;
use crate::layout::ObjectLayout;
use crate::layout::PreludeLayout;
use crate::layout::SyntheticSymbolsLayout;
use crate::output_section_id::SectionOutputInfo;
use crate::output_section_map::OutputSectionMap;
use crate::output_section_part_map::OutputSectionPartMap;
use crate::output_trace::TraceOutput;
use crate::platform::Arch;
use crate::timing_phase;
use crate::verbose_timing_phase;
use object::LittleEndian;
use object::from_bytes_mut;
use object::write::WritableBuffer;
use rayon::iter::IndexedParallelIterator as _;
use rayon::iter::IntoParallelIterator as _;
use rayon::iter::ParallelIterator as _;
use std::sync::atomic::Ordering::Relaxed;

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
    _phantom: std::marker::PhantomData<(&'layout (), &'out ())>,
}

impl<'layout, 'out> TableWriter<'layout, 'out> {
    fn from_layout(
        _layout: &'layout CoffLayout,
        buffers: &mut OutputSectionPartMap<&'out mut [u8]>,
    ) -> TableWriter<'layout, 'out> {
        Self::new(buffers)
    }

    fn new(_buffers: &mut OutputSectionPartMap<&'out mut [u8]>) -> TableWriter<'layout, 'out> {
        TableWriter {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Checks that we used all the entries that we requested during layout.
    fn validate_empty(&self, _mem_sizes: &OutputSectionPartMap<u64>) -> Result {
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
    _prelude: &PreludeLayout<Coff>,
    buffers: &mut OutputSectionPartMap<&mut [u8]>,
    _table_writer: &mut TableWriter<'_, '_>,
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

    Ok(())
}

fn populate_dos_header<A: Arch<Platform = Coff>>(_layout: &CoffLayout, header: &mut DosHeader) {
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
    header.e_lfanew.set(e, 0_u32);
}

fn write_dos_stub<A: Arch<Platform = Coff>>(mut bytes: &mut [u8]) {
    bytes.write_bytes(coff::DOS_STUB);
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
