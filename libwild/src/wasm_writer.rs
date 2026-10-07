use crate::OutputFileData;
use crate::bail;
use crate::ensure;
use crate::error::Context as _;
use crate::error::Result;
use crate::file_writer::SizedOutput;
use crate::file_writer::split_output_into_sections;
use crate::layout::Layout;
use crate::platform::Arch;
use crate::timing_phase;
use crate::verbose_timing_phase;
use crate::wasm::TARGET_FEATURE_PREFIX_DISALLOWED;
use crate::wasm::TARGET_FEATURE_PREFIX_USED;
use crate::wasm::TARGET_FEATURES_SECTION_NAME;
use crate::wasm::WASM_DEAD_INDEX;
use crate::wasm::WASM_MAGIC;
use crate::wasm::WASM_VERSION;
use crate::wasm::Wasm;
use crate::wasm::WasmDataSegmentLayout;
use crate::wasm::WasmFunctionBody;
use crate::wasm::WasmGotMemSource;
use crate::wasm::WasmInputTargetFeature;
use crate::wasm::WasmLayout;
use crate::wasm::WasmObjectIndexMap;
use crate::wasm::WasmRelocation;
use crate::wasm::WasmSymbol;
use crate::wasm::WasmSymbolKind;
use crate::wasm::count_output_imports;
use crate::wasm::demangle_symbol_name;
use crate::wasm::output_data_has_passive;
use crate::wasm::output_data_segment_count;
use crate::wasm::output_section_id;
use crate::wasm::reject_tls_template_reloc;
use crate::wasm::relocation_type_to_string;
use crate::wasm::section_id;
use crate::wasm::wasm_symbol_name_str;
use hashbrown::HashMap;
use hashbrown::HashSet;
use leb128::write::unsigned_len as uleb128_size;
use rayon::prelude::*;
use std::borrow::Cow;
use std::ops::Range;
use wasm_encoder::ConstExpr;
use wasm_encoder::ElementSection;
use wasm_encoder::Elements;
use wasm_encoder::ExportSection;
use wasm_encoder::FunctionSection;
use wasm_encoder::GlobalSection;
use wasm_encoder::ImportSection;
use wasm_encoder::MemorySection;
use wasm_encoder::NameMap;
use wasm_encoder::NameSection;
use wasm_encoder::TableSection;
use wasm_encoder::TypeSection;
use wasmparser::RelocationType;

/// Write `value` as an unsigned LEB128 into `buf`, returning the number of bytes written.
pub(crate) fn write_uleb128(buf: &mut [u8], value: u64) -> usize {
    let mut writable = &mut *buf;
    leb128::write::unsigned(&mut writable, value).unwrap()
}

/// Write `value` as a signed LEB128 into `buf`, returning the number of bytes written.
pub(crate) fn write_sleb128(buf: &mut [u8], value: i64) -> usize {
    let mut writable = &mut *buf;
    leb128::write::signed(&mut writable, value).unwrap()
}

/// Write `value` as a 5-byte fixed-width unsigned LEB128. Used for wasm reloc slots that reserve
/// exactly 5 bytes regardless of the encoded value.
fn write_uleb128_5(buf: &mut [u8; 5], value: u32) {
    buf[0] = (value as u8 & 0x7f) | 0x80;
    buf[1] = ((value >> 7) as u8 & 0x7f) | 0x80;
    buf[2] = ((value >> 14) as u8 & 0x7f) | 0x80;
    buf[3] = ((value >> 21) as u8 & 0x7f) | 0x80;
    buf[4] = (value >> 28) as u8 & 0x0f;
}

/// Write `value` as a 5-byte fixed-width signed LEB128. The high three bits of the final byte are
/// sign-extended so the encoded form is canonical for any `i32`.
fn write_sleb128_5(buf: &mut [u8; 5], value: i32) {
    let v = value as u32;
    buf[0] = (v as u8 & 0x7f) | 0x80;
    buf[1] = ((v >> 7) as u8 & 0x7f) | 0x80;
    buf[2] = ((v >> 14) as u8 & 0x7f) | 0x80;
    buf[3] = ((v >> 21) as u8 & 0x7f) | 0x80;
    let last = (v >> 28) as u8 & 0x0f;
    let sign_ext = if value < 0 { 0x70 } else { 0x00 };
    buf[4] = last | sign_ext;
}

pub(crate) fn apply_relocation(
    bytes: &mut [u8],
    reloc: &WasmRelocation,
    value: u32,
) -> crate::error::Result<()> {
    let offset = reloc.offset as usize;
    let size = reloc.slot_size();
    let end = offset
        .checked_add(size)
        .context("Wasm relocation offset overflow")?;
    let slot = bytes
        .get_mut(offset..end)
        .context("Wasm relocation slot out of range")?;
    match reloc.ty {
        RelocationType::FunctionIndexLeb
        | RelocationType::MemoryAddrLeb
        | RelocationType::TypeIndexLeb
        | RelocationType::GlobalIndexLeb
        | RelocationType::EventIndexLeb
        | RelocationType::TableNumberLeb => {
            let buf: &mut [u8; 5] = slot.try_into().expect("slot_size returned 5");
            write_uleb128_5(buf, value);
        }
        RelocationType::TableIndexSleb
        | RelocationType::TableIndexRelSleb
        | RelocationType::MemoryAddrSleb
        | RelocationType::MemoryAddrRelSleb
        | RelocationType::MemoryAddrTlsSleb => {
            let buf: &mut [u8; 5] = slot.try_into().expect("slot_size returned 5");
            write_sleb128_5(buf, value as i32);
        }
        RelocationType::TableIndexI32
        | RelocationType::MemoryAddrI32
        | RelocationType::FunctionOffsetI32
        | RelocationType::SectionOffsetI32
        | RelocationType::GlobalIndexI32
        | RelocationType::FunctionIndexI32 => {
            slot.copy_from_slice(&value.to_le_bytes());
        }
        other => bail!(
            "unsupported Wasm relocation type {}",
            relocation_type_to_string(other)
        ),
    }
    Ok(())
}

fn reloc_value_with_addend(base: u32, addend: i64) -> Result<u32> {
    let value = i64::from(base)
        .checked_add(addend)
        .context("Wasm relocation value overflow")?;
    u32::try_from(value).context("Wasm relocation value out of range")
}

/// Apply addend policy. Relative table/memory bases already include the addend.
fn finalize_reloc_value(reloc: &WasmRelocation, base: u32) -> Result<u32> {
    if matches!(
        reloc.ty,
        RelocationType::MemoryAddrRelSleb
            | RelocationType::TableIndexRelSleb
            | RelocationType::MemoryAddrTlsSleb
    ) {
        Ok(base)
    } else {
        reloc_value_with_addend(base, reloc.addend)
    }
}

fn apply_resolved_reloc(
    index_map: &WasmObjectIndexMap,
    reloc: &WasmRelocation,
    symbols: &[WasmSymbol],
    function_table_slots: &[u32],
    memory_base: u32,
    tls_base: u32,
    buf: &mut [u8],
) -> Result<()> {
    let base =
        index_map.resolve_reloc(reloc, symbols, function_table_slots, memory_base, tls_base)?;
    apply_relocation(buf, reloc, finalize_reloc_value(reloc, base)?)
}

fn relocs_in_index_range(relocs: &[WasmRelocation], range: Range<u32>) -> &[WasmRelocation] {
    let start = range.start as usize;
    let end = range.end as usize;
    relocs.get(start..end).unwrap_or(&[])
}

fn apply_section_reloc(
    index_map: &WasmObjectIndexMap,
    reloc: &WasmRelocation,
    local_base: u32,
    symbols: &[WasmSymbol],
    function_table_slots: &[u32],
    memory_base: u32,
    tls_base: u32,
    in_tls_template: bool,
    buf: &mut [u8],
) -> Result<()> {
    let mut reloc = *reloc;
    reloc.offset = reloc
        .offset
        .checked_sub(local_base)
        .context("Wasm relocation offset is before the body or payload start")?;
    if in_tls_template {
        reject_tls_template_reloc(index_map, &reloc)?;
    }
    apply_resolved_reloc(
        index_map,
        &reloc,
        symbols,
        function_table_slots,
        memory_base,
        tls_base,
        buf,
    )
}

pub(crate) fn write<'data, A: Arch<Platform = Wasm>>(
    sized_output: &mut SizedOutput<impl OutputFileData>,
    layout: &Layout<'data, Wasm>,
) -> Result<()> {
    timing_phase!("Write Wasm output");
    let (mut section_buffers, mut padding) =
        split_output_into_sections(layout, &mut sized_output.out);
    padding.fill_zero();

    let preamble = section_buffers
        .get_mut(crate::output_section_id::FILE_HEADER)
        .get_mut(..8)
        .context("Wasm output buffer is shorter than the 8-byte preamble")?;
    preamble[..4].copy_from_slice(&WASM_MAGIC);
    preamble[4..8].copy_from_slice(&WASM_VERSION.to_le_bytes());

    if let Some(unsupported) = layout.format_specific.unsupported_output.first() {
        bail!("Wasm {unsupported} emission is not implemented yet");
    }

    {
        timing_phase!("Copy Wasm metadata sections");
        write_metadata_sections(&layout.format_specific, &mut section_buffers)?;
    }
    {
        timing_phase!("Write Wasm code section");
        write_code_section(
            &layout.format_specific,
            section_buffers.get_mut(output_section_id::WASM_CODE),
        )?;
    }
    {
        timing_phase!("Write Wasm data section");
        write_data_section(
            &layout.format_specific,
            section_buffers.get_mut(output_section_id::WASM_DATA),
        )?;
    }

    Ok(())
}

fn write_metadata_sections(
    layout: &WasmLayout<'_>,
    section_buffers: &mut crate::output_section_map::OutputSectionMap<&mut [u8]>,
) -> Result<()> {
    let encoded = &layout.encoded_metadata;
    copy_encoded_section(
        encoded.ty.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_TYPE),
    )?;
    copy_encoded_section(
        encoded.import.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_IMPORT),
    )?;
    copy_encoded_section(
        encoded.function.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_FUNCTION),
    )?;
    copy_encoded_section(
        encoded.table.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_TABLE),
    )?;
    copy_encoded_section(
        encoded.memory.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_MEMORY),
    )?;
    copy_encoded_section(
        encoded.global.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_GLOBAL),
    )?;
    copy_encoded_section(
        encoded.export.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_EXPORT),
    )?;
    copy_encoded_section(
        encoded.element.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_ELEMENT),
    )?;
    copy_encoded_section(
        encoded.data_count.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_DATA_COUNT),
    )?;
    copy_encoded_section(
        encoded.name.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_NAME),
    )?;
    copy_encoded_section(
        encoded.target_features.as_ref(),
        section_buffers.get_mut(output_section_id::WASM_TARGET_FEATURES),
    )?;
    Ok(())
}

fn copy_encoded_section(encoded: Option<&Vec<u8>>, out: &mut [u8]) -> Result<()> {
    match encoded {
        Some(encoded) => {
            ensure!(
                out.len() == encoded.len(),
                "Wasm metadata section size allocated {}, encoded {}",
                out.len(),
                encoded.len()
            );
            out.copy_from_slice(encoded);
        }
        None => {
            ensure!(
                out.is_empty(),
                "Wasm metadata section unexpectedly allocated {} bytes",
                out.len()
            );
        }
    }
    Ok(())
}

// Each `WasmFunctionBody.bytes` is the raw body content (locals + operators) without a size prefix.
fn write_code_section(wasm_layout: &WasmLayout<'_>, out: &mut [u8]) -> Result<()> {
    let bodies = &wasm_layout.function_bodies;
    let object_index_maps = &wasm_layout.object_index_maps;
    let object_code_relocations = &wasm_layout.object_code_relocations;
    let per_object_symbols = &wasm_layout.per_object_symbols;
    let function_table_slots = &wasm_layout.function_table_slots;
    let memory_base = wasm_layout.memory_base;
    let tls_base = wasm_layout.tls_base;

    if bodies.is_empty() {
        ensure!(
            out.is_empty(),
            "Wasm code section buffer is {} bytes but no bodies to write",
            out.len()
        );
        return Ok(());
    }

    let mut pos = 0;

    // Section id.
    out[pos] = section_id::CODE;
    pos += 1;

    let count = bodies.len() as u64;
    let count_leb_size = uleb128_size(count);
    let bodies_with_prefix_total: usize = bodies
        .iter()
        .map(|b| {
            let body_len = b.bytes.len() as u64;
            uleb128_size(body_len) + b.bytes.len()
        })
        .sum();
    let payload_size = (count_leb_size + bodies_with_prefix_total) as u64;

    pos += write_uleb128(&mut out[pos..], payload_size);
    pos += write_uleb128(&mut out[pos..], count);
    let bodies_region_start = pos;

    // Split the body region into non-overlapping slots, then emit in parallel.
    let mut body_slots: Vec<(&mut [u8], &WasmFunctionBody<'_>)> = Vec::with_capacity(bodies.len());
    {
        verbose_timing_phase!("Split Wasm code body slots");
        let mut rest = &mut out[bodies_region_start..];
        for body in bodies {
            let body_len = body.bytes.len() as u64;
            let slot_len = uleb128_size(body_len) + body.bytes.len();
            ensure!(
                rest.len() >= slot_len,
                "Wasm code section body slot overflow (need {slot_len}, have {})",
                rest.len()
            );
            let (slot, next) = rest.split_at_mut(slot_len);
            body_slots.push((slot, body));
            rest = next;
        }
        ensure!(
            rest.is_empty(),
            "Wasm code section has {} trailing unused bytes after body slots",
            rest.len()
        );
    }

    {
        verbose_timing_phase!("Emit Wasm code bodies");
        body_slots
            .into_par_iter()
            .try_for_each(|(slot, body)| -> Result<()> {
                verbose_timing_phase!("Emit Wasm code body");
                let body_len = body.bytes.len() as u64;
                let pos = write_uleb128(slot, body_len);
                let len = body.bytes.len();
                slot[pos..pos + len].copy_from_slice(&body.bytes);
                let body_bytes = &mut slot[pos..pos + len];
                let index_map = &object_index_maps[body.object_index];
                let symbols = &per_object_symbols[body.object_index];
                let object_relocs = object_code_relocations
                    .get(body.object_index)
                    .map_or(&[][..], Vec::as_slice);
                for reloc in relocs_in_index_range(object_relocs, body.reloc_range.clone()) {
                    apply_section_reloc(
                        index_map,
                        reloc,
                        body.code_offset,
                        symbols,
                        function_table_slots,
                        memory_base,
                        tls_base,
                        false,
                        body_bytes,
                    )?;
                }
                Ok(())
            })?;
    }

    Ok(())
}

fn write_data_section(wasm_layout: &WasmLayout<'_>, out: &mut [u8]) -> Result<()> {
    let object_data_layouts = &wasm_layout.object_data_layouts;
    let segment_count: u32 = object_data_layouts
        .iter()
        .map(|obj| u32::try_from(obj.len()).unwrap_or(u32::MAX))
        .sum();
    if segment_count == 0 {
        ensure!(
            out.is_empty(),
            "Wasm data section buffer is {} bytes but no segments to write",
            out.len()
        );
        return Ok(());
    }

    // Flatten (object_index, segment) for parallel emit. Header is serial.
    let flat: Vec<(usize, &WasmDataSegmentLayout<'_>)> = object_data_layouts
        .iter()
        .enumerate()
        .flat_map(|(obj_idx, segs)| segs.iter().map(move |seg| (obj_idx, seg)))
        .collect();
    ensure!(
        flat.len() == segment_count as usize,
        "Wasm data segment count mismatch"
    );

    let mut pos = 0;
    out[pos] = section_id::DATA;
    pos += 1;

    let segments_total: u64 = flat
        .iter()
        .map(|(_, seg)| u64::from(seg.encoded_output_size))
        .sum();
    let count_leb_size = uleb128_size(u64::from(segment_count));
    let payload_size = count_leb_size as u64 + segments_total;
    pos += write_uleb128(&mut out[pos..], payload_size);
    pos += write_uleb128(&mut out[pos..], u64::from(segment_count));
    let segments_region_start = pos;

    let object_index_maps = &wasm_layout.object_index_maps;
    let object_data_relocations = &wasm_layout.object_data_relocations;
    let per_object_symbols = &wasm_layout.per_object_symbols;
    let function_table_slots = &wasm_layout.function_table_slots;
    let memory_base = wasm_layout.memory_base;
    let tls_base = wasm_layout.tls_base;

    let mut segment_slots: Vec<(&mut [u8], usize, &WasmDataSegmentLayout<'_>)> =
        Vec::with_capacity(flat.len());
    {
        verbose_timing_phase!("Split Wasm data segment slots");
        let mut rest = &mut out[segments_region_start..];
        for (obj_idx, segment) in flat {
            let slot_len = segment.encoded_output_size as usize;
            ensure!(
                rest.len() >= slot_len,
                "Wasm data section segment slot overflow (need {slot_len}, have {})",
                rest.len()
            );
            let (slot, next) = rest.split_at_mut(slot_len);
            segment_slots.push((slot, obj_idx, segment));
            rest = next;
        }
        ensure!(
            rest.is_empty(),
            "Wasm data section has {} trailing unused bytes after segment slots",
            rest.len()
        );
    }

    {
        verbose_timing_phase!("Emit Wasm data segments");
        segment_slots
            .into_par_iter()
            .try_for_each(|(slot, obj_idx, segment)| -> Result<()> {
                verbose_timing_phase!("Emit Wasm data segment");
                write_data_segment(
                    slot,
                    segment,
                    &object_index_maps[obj_idx],
                    object_data_relocations
                        .get(obj_idx)
                        .map_or(&[][..], Vec::as_slice),
                    per_object_symbols[obj_idx],
                    function_table_slots,
                    memory_base,
                    tls_base,
                )
            })?;
    }

    Ok(())
}

fn write_data_segment(
    out: &mut [u8],
    segment: &WasmDataSegmentLayout<'_>,
    index_map: &WasmObjectIndexMap,
    object_relocs: &[WasmRelocation],
    symbols: &[WasmSymbol],
    function_table_slots: &[u32],
    memory_base: u32,
    tls_base: u32,
) -> Result<()> {
    ensure!(
        out.len() == segment.encoded_output_size as usize,
        "Wasm data segment buffer size {} != encoded_output_size {}",
        out.len(),
        segment.encoded_output_size
    );

    let mut pos = 0;
    if segment.passive {
        out[pos] = 0x01;
        pos += 1;
    } else if segment.output_memory_index == 0 {
        out[pos] = 0x00;
        pos += 1;
        pos = write_active_offset_expr(out, pos, segment.output_memory_offset)?;
    } else {
        out[pos] = 0x02;
        pos += 1;
        pos += write_uleb128(&mut out[pos..], u64::from(segment.output_memory_index));
        pos = write_active_offset_expr(out, pos, segment.output_memory_offset)?;
    }

    let data_len = segment.data.len() as u64;
    pos += write_uleb128(&mut out[pos..], data_len);
    let payload = &mut out[pos..pos + segment.data.len()];
    payload.copy_from_slice(segment.data);
    for reloc in relocs_in_index_range(object_relocs, segment.reloc_range.clone()) {
        apply_section_reloc(
            index_map,
            reloc,
            segment.payload_start,
            symbols,
            function_table_slots,
            memory_base,
            tls_base,
            segment.tls_template,
            payload,
        )?;
    }
    pos += segment.data.len();

    ensure!(
        pos == out.len(),
        "Wasm data segment wrote {pos} bytes but buffer is {} bytes",
        out.len()
    );
    Ok(())
}

fn write_active_offset_expr(out: &mut [u8], mut pos: usize, offset: u32) -> Result<usize> {
    out[pos] = 0x41;
    pos += 1;
    let offset_i32 = i32::try_from(offset)
        .with_context(|| format!("Wasm data segment memory offset {offset}"))?;
    pos += write_sleb128(&mut out[pos..], i64::from(offset_i32));
    out[pos] = 0x0b;
    pos += 1;
    Ok(pos)
}

/// Build a `type` section from a list of function types in output order. Callers must have
/// already done dedup across input modules.
pub(crate) fn build_type_section(types: &[wasmparser::FuncType]) -> Result<TypeSection> {
    let mut section = TypeSection::new();
    for ty in types {
        let params: Vec<wasm_encoder::ValType> = ty
            .params()
            .iter()
            .copied()
            .map(convert_val_type)
            .collect::<Result<_>>()?;
        let results: Vec<wasm_encoder::ValType> = ty
            .results()
            .iter()
            .copied()
            .map(convert_val_type)
            .collect::<Result<_>>()?;
        section.ty().function(params, results);
    }
    Ok(section)
}

/// Build an `import` section. `type_index` for function imports must be the output type index.
pub(crate) fn build_import_section(imports: &[OutputImport<'_>]) -> Result<ImportSection> {
    let mut section = ImportSection::new();
    for import in imports {
        let entity = match import.entity {
            OutputImportEntity::Function { type_index } => {
                wasm_encoder::EntityType::Function(type_index)
            }
            OutputImportEntity::Global(ty) => {
                wasm_encoder::EntityType::Global(convert_global_type(ty)?)
            }
            OutputImportEntity::Memory(ty) => {
                wasm_encoder::EntityType::Memory(convert_memory_type(ty))
            }
        };
        section.import(import.module, import.name, entity);
    }
    Ok(section)
}

/// Build a `function` section. Each entry is the (output) type index of a module-defined
/// function, in `code` section order.
pub(crate) fn build_function_section(type_indices: &[u32]) -> FunctionSection {
    let mut section = FunctionSection::new();
    for &type_index in type_indices {
        section.function(type_index);
    }
    section
}

/// Build a `global` section from `(type, init_expr_bytes)` pairs. The init-expr bytes are the
/// raw const-expression bytes from the input *without* the trailing `end` opcode; the encoder
/// re-appends `end` itself.
pub(crate) fn build_global_section(globals: &[OutputGlobal<'_>]) -> Result<GlobalSection> {
    let mut section = GlobalSection::new();
    for global in globals {
        let init_expr = wasm_encoder::ConstExpr::raw(global.init_expr_body.iter().copied());
        section.global(convert_global_type(global.ty)?, &init_expr);
    }
    Ok(section)
}

pub(crate) fn build_table_section(tables: &[wasmparser::TableType]) -> Result<TableSection> {
    let mut section = TableSection::new();
    for &table in tables {
        ensure!(
            table.element_type.is_func_ref(),
            "only funcref tables are supported (got {:?})",
            table.element_type
        );
        section.table(wasm_encoder::TableType {
            element_type: wasm_encoder::RefType::FUNCREF,
            minimum: table.initial,
            maximum: table.maximum,
            table64: table.table64,
            shared: table.shared,
        });
    }
    Ok(section)
}

/// One active element segment on table 0 at offset 1 (functions occupy slots 1..).
pub(crate) fn build_element_section(element_functions: &[u32]) -> ElementSection {
    let mut section = ElementSection::new();
    if element_functions.is_empty() {
        return section;
    }
    let offset = ConstExpr::i32_const(1);
    section.active(
        Some(0),
        &offset,
        Elements::Functions(std::borrow::Cow::Borrowed(element_functions)),
    );
    section
}

pub(crate) fn build_memory_section(memories: &[wasmparser::MemoryType]) -> MemorySection {
    let mut section = MemorySection::new();
    for &memory in memories {
        section.memory(convert_memory_type(memory));
    }
    section
}

/// Build an `export` section.
pub(crate) fn build_export_section(exports: &[OutputExport<'_>]) -> ExportSection {
    let mut section = ExportSection::new();
    for export in exports {
        section.export(export.name, convert_export_kind(export.kind), export.index);
    }
    section
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct OutputImport<'a> {
    pub(crate) module: &'a str,
    pub(crate) name: &'a str,
    pub(crate) entity: OutputImportEntity,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum OutputImportEntity {
    Function { type_index: u32 },
    Global(wasmparser::GlobalType),
    Memory(wasmparser::MemoryType),
}

#[derive(Debug, Clone)]
pub(crate) struct OutputGlobal<'a> {
    pub(crate) ty: wasmparser::GlobalType,
    /// Const-expression body without the trailing `end` opcode.
    pub(crate) init_expr_body: Cow<'a, [u8]>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct OutputExport<'a> {
    pub(crate) name: &'a str,
    pub(crate) kind: wasmparser::ExternalKind,
    pub(crate) index: u32,
}

/// Strip the trailing `end` (0x0b) opcode from a wasmparser-parsed const expression so the
/// bytes are suitable for `wasm_encoder::ConstExpr::raw`. Returns `None` if the buffer doesn't
/// terminate with `end`, which would indicate a malformed input.
pub(crate) fn const_expr_body<'a>(expr: &wasmparser::ConstExpr<'a>) -> Option<&'a [u8]> {
    let mut reader = expr.get_binary_reader();
    let n = reader.bytes_remaining();
    let bytes = reader.read_bytes(n).ok()?;
    bytes.strip_suffix(&[0x0b])
}

fn convert_val_type(t: wasmparser::ValType) -> Result<wasm_encoder::ValType> {
    Ok(match t {
        wasmparser::ValType::I32 => wasm_encoder::ValType::I32,
        wasmparser::ValType::I64 => wasm_encoder::ValType::I64,
        wasmparser::ValType::F32 => wasm_encoder::ValType::F32,
        wasmparser::ValType::F64 => wasm_encoder::ValType::F64,
        wasmparser::ValType::V128 => bail!("V128 value type is not supported yet"),
        wasmparser::ValType::Ref(_) => bail!("reference value types are not supported yet"),
    })
}

fn convert_global_type(t: wasmparser::GlobalType) -> Result<wasm_encoder::GlobalType> {
    Ok(wasm_encoder::GlobalType {
        val_type: convert_val_type(t.content_type)?,
        mutable: t.mutable,
        shared: t.shared,
    })
}

fn convert_memory_type(t: wasmparser::MemoryType) -> wasm_encoder::MemoryType {
    wasm_encoder::MemoryType {
        minimum: t.initial,
        maximum: t.maximum,
        memory64: t.memory64,
        shared: t.shared,
        page_size_log2: t.page_size_log2,
    }
}

fn convert_export_kind(k: wasmparser::ExternalKind) -> wasm_encoder::ExportKind {
    match k {
        wasmparser::ExternalKind::Func | wasmparser::ExternalKind::FuncExact => {
            wasm_encoder::ExportKind::Func
        }
        wasmparser::ExternalKind::Table => wasm_encoder::ExportKind::Table,
        wasmparser::ExternalKind::Memory => wasm_encoder::ExportKind::Memory,
        wasmparser::ExternalKind::Global => wasm_encoder::ExportKind::Global,
        wasmparser::ExternalKind::Tag => wasm_encoder::ExportKind::Tag,
    }
}

#[derive(Debug, Default)]
pub(crate) struct EncodedMetadata {
    ty: Option<Vec<u8>>,
    import: Option<Vec<u8>>,
    function: Option<Vec<u8>>,
    global: Option<Vec<u8>>,
    export: Option<Vec<u8>>,
    memory: Option<Vec<u8>>,
    table: Option<Vec<u8>>,
    element: Option<Vec<u8>>,
    data_count: Option<Vec<u8>>,
    name: Option<Vec<u8>>,
    target_features: Option<Vec<u8>>,
}

impl EncodedMetadata {
    pub(crate) fn add_sizes_to(
        &self,
        sizes: &mut crate::output_section_part_map::OutputSectionPartMap<u64>,
    ) {
        add_encoded_section_size(sizes, crate::wasm::part_id::WASM_TYPE, self.ty.as_ref());
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_IMPORT,
            self.import.as_ref(),
        );
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_FUNCTION,
            self.function.as_ref(),
        );
        add_encoded_section_size(sizes, crate::wasm::part_id::WASM_TABLE, self.table.as_ref());
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_MEMORY,
            self.memory.as_ref(),
        );
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_GLOBAL,
            self.global.as_ref(),
        );
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_EXPORT,
            self.export.as_ref(),
        );
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_ELEMENT,
            self.element.as_ref(),
        );
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_DATA_COUNT,
            self.data_count.as_ref(),
        );
        add_encoded_section_size(sizes, crate::wasm::part_id::WASM_NAME, self.name.as_ref());
        add_encoded_section_size(
            sizes,
            crate::wasm::part_id::WASM_TARGET_FEATURES,
            self.target_features.as_ref(),
        );
    }
}

fn add_encoded_section_size(
    sizes: &mut crate::output_section_part_map::OutputSectionPartMap<u64>,
    part_id: crate::part_id::PartId,
    section: Option<&Vec<u8>>,
) {
    if let Some(bytes) = section {
        sizes.increment(part_id, bytes.len() as u64);
    }
}

pub(crate) fn encode_metadata_sections(layout: &WasmLayout<'_>) -> Result<EncodedMetadata> {
    timing_phase!("Encode Wasm metadata sections");
    let mut encoded = EncodedMetadata::default();

    {
        timing_phase!("Encode Wasm type section");
        let type_section = build_type_section(&layout.output_types)?;
        if !type_section.is_empty() {
            encoded.ty = Some(encode_wasm_section(&type_section));
        }
    }

    {
        timing_phase!("Encode Wasm import section");
        let import_section = build_import_section(&layout.imports)?;
        if !import_section.is_empty() {
            encoded.import = Some(encode_wasm_section(&import_section));
        }
    }

    {
        timing_phase!("Encode Wasm function section");
        let function_section = build_function_section(&layout.function_type_indices);
        if !function_section.is_empty() {
            encoded.function = Some(encode_wasm_section(&function_section));
        }
    }

    {
        timing_phase!("Encode Wasm global section");
        let global_section = build_global_section(&layout.globals)?;
        if !global_section.is_empty() {
            encoded.global = Some(encode_wasm_section(&global_section));
        }
    }

    {
        timing_phase!("Encode Wasm export section");
        let export_section = build_export_section(&layout.exports);
        if !export_section.is_empty() {
            encoded.export = Some(encode_wasm_section(&export_section));
        }
    }

    {
        timing_phase!("Encode Wasm memory section");
        let memory_section = build_memory_section(&layout.memories);
        if !memory_section.is_empty() {
            encoded.memory = Some(encode_wasm_section(&memory_section));
        }
    }

    if !layout.tables.is_empty() {
        timing_phase!("Encode Wasm table section");
        let table_section = build_table_section(&layout.tables)?;
        encoded.table = Some(encode_wasm_section(&table_section));
    }

    if !layout.element_functions.is_empty() {
        timing_phase!("Encode Wasm element section");
        let element_section = build_element_section(&layout.element_functions);
        encoded.element = Some(encode_wasm_section(&element_section));
    }

    {
        timing_phase!("Encode Wasm name section");
        if let Some(name_section) = build_name_section(layout) {
            encoded.name = Some(encode_wasm_section(&name_section));
        }
    }

    {
        timing_phase!("Encode Wasm target_features section");
        if let Some(target_features) =
            build_target_features_section(&layout.target_feature_inputs, layout.extra_features)?
        {
            encoded.target_features = Some(encode_wasm_section(&target_features));
        }
    }

    {
        timing_phase!("Encode Wasm data count section");
        // Validators reject `memory.init` / `data.drop` without this section. A remaining
        // passive segment emits it too.
        if layout.code_references_data_segment
            || output_data_has_passive(&layout.object_data_layouts)
        {
            let count = output_data_segment_count(&layout.object_data_layouts);
            encoded.data_count = Some(encode_wasm_section(&wasm_encoder::DataCountSection {
                count,
            }));
        }
    }

    Ok(encoded)
}

fn encode_wasm_section(section: &impl wasm_encoder::Section) -> Vec<u8> {
    let mut bytes = Vec::new();
    section.append_to(&mut bytes);
    bytes
}

/// Per-object name entries.
#[derive(Default)]
struct ObjectNameEntries<'a> {
    functions: Vec<(u32, &'a str)>,
    globals: Vec<(u32, &'a str)>,
}

fn build_name_section(layout: &WasmLayout<'_>) -> Option<NameSection> {
    let names = &layout.name_inputs;
    let (n_func_imports, n_global_imports) = count_output_imports(layout);
    let n_funcs = n_func_imports + layout.function_type_indices.len();
    let n_globals = n_global_imports + layout.globals.len();
    let mut function_names: Vec<Option<&str>> = vec![None; n_funcs];
    let mut global_names: Vec<Option<&str>> = vec![None; n_globals];
    let mut got_mem_names: Vec<String> = Vec::new();
    let mut got_func_names: Vec<String> = Vec::new();

    // Host / remaining imports.
    let mut next_func_import = 0u32;
    let mut next_global_import = 0u32;
    for import in &layout.imports {
        match import.entity {
            OutputImportEntity::Function { .. } => {
                set_name_first_wins(&mut function_names, next_func_import, import.name);
                next_func_import += 1;
            }
            OutputImportEntity::Global(_) => {
                set_name_first_wins(&mut global_names, next_global_import, import.name);
                next_global_import += 1;
            }
            OutputImportEntity::Memory(_) => {}
        }
    }

    // Linker-synthesised functions / globals.
    if let Some(idx) = names.memory_base_global {
        set_name_first_wins(&mut global_names, idx, "__memory_base");
    }
    if let Some(idx) = names.table_base_global {
        set_name_first_wins(&mut global_names, idx, "__table_base");
    }
    if let Some(idx) = names.stack_pointer_global {
        set_name_first_wins(&mut global_names, idx, "__stack_pointer");
    }
    if let Some(idx) = names.tls_base_global {
        set_name_first_wins(&mut global_names, idx, "__tls_base");
    }
    if let Some(idx) = names.tls_size_global {
        set_name_first_wins(&mut global_names, idx, "__tls_size");
    }
    if let Some(idx) = names.tls_align_global {
        set_name_first_wins(&mut global_names, idx, "__tls_align");
    }
    for &(known, idx) in &names.data_address_globals {
        set_name_first_wins(&mut global_names, idx, <&str>::from(known));
    }
    if let Some(got_base) = names.got_mem_global_base {
        got_mem_names.reserve(names.got_mem.len());
        for (i, source) in names.got_mem.iter().enumerate() {
            let name = match *source {
                WasmGotMemSource::Symbol(Some(sym)) => format!(
                    "GOT.data.internal.{}",
                    demangle_symbol_name(sym, names.demangle)
                ),
                WasmGotMemSource::Symbol(None) => format!("GOT.data.internal.{i}"),
                WasmGotMemSource::LinkerDefined(known) => {
                    let sym = std::str::from_utf8(known.name()).unwrap_or("?");
                    format!("GOT.data.internal.{sym}")
                }
            };
            got_mem_names.push(name);
        }
        for (i, name) in got_mem_names.iter().enumerate() {
            set_name_first_wins(&mut global_names, got_base + i as u32, name.as_str());
        }
    }
    if let Some(got_base) = names.got_func_global_base {
        got_func_names.reserve(names.got_func_names.len());
        for (i, name) in names.got_func_names.iter().enumerate() {
            got_func_names.push(match name {
                Some(name) => format!(
                    "GOT.func.internal.{}",
                    demangle_symbol_name(name, names.demangle)
                ),
                None => format!("GOT.func.internal.{i}"),
            });
        }
        for (i, name) in got_func_names.iter().enumerate() {
            set_name_first_wins(&mut global_names, got_base + i as u32, name.as_str());
        }
    }
    if let Some(idx) = names.call_ctors_func {
        set_name_first_wins(&mut function_names, idx, "__wasm_call_ctors");
    }
    if let Some(idx) = names.init_tls_func {
        set_name_first_wins(&mut function_names, idx, "__wasm_init_tls");
    }

    let per_object_names: Vec<ObjectNameEntries<'_>> = layout
        .per_object_data
        .par_iter()
        .zip(layout.per_object_symbols.par_iter())
        .zip(layout.object_index_maps.par_iter())
        .map(|((data, symbols), index_map)| {
            verbose_timing_phase!("Collect Wasm object name entries");
            let mut entries = ObjectNameEntries::default();
            for sym in *symbols {
                let Some(name) = wasm_symbol_name_str(data, sym) else {
                    continue;
                };
                match sym.kind {
                    WasmSymbolKind::Func
                        if let Some(&out_idx) =
                            index_map.function_indices.get(sym.index as usize)
                            && out_idx != WASM_DEAD_INDEX =>
                    {
                        entries.functions.push((out_idx, name));
                    }
                    WasmSymbolKind::Global
                        if let Some(&out_idx) =
                            index_map.global_indices.get(sym.index as usize)
                            && out_idx != WASM_DEAD_INDEX =>
                    {
                        entries.globals.push((out_idx, name));
                    }
                    _ => {}
                }
            }
            entries
        })
        .collect();
    for entries in per_object_names {
        for (out_idx, name) in entries.functions {
            set_name_first_wins(&mut function_names, out_idx, name);
        }
        for (out_idx, name) in entries.globals {
            set_name_first_wins(&mut global_names, out_idx, name);
        }
    }

    // Named after object symbols so first-wins keeps `{export}.command_export` rather than the
    // export name that is retargeted onto the wrapper.
    for (idx, name) in &layout.command_export_wrapper_names {
        set_name_first_wins(&mut function_names, *idx, name.as_str());
    }

    for export in &layout.exports {
        match export.kind {
            wasmparser::ExternalKind::Func => {
                set_name_first_wins(&mut function_names, export.index, export.name);
            }
            wasmparser::ExternalKind::Global => {
                set_name_first_wins(&mut global_names, export.index, export.name);
            }
            _ => {}
        }
    }

    let function_map = name_map_from_dense(&function_names, names.demangle);
    let global_map = name_map_from_dense(&global_names, names.demangle);
    if function_map.is_none() && global_map.is_none() {
        return None;
    }

    let mut section = NameSection::new();
    if let Some(map) = function_map {
        section.functions(&map);
    }
    if let Some(map) = global_map {
        section.globals(&map);
    }
    Some(section)
}

fn set_name_first_wins<'a>(names: &mut Vec<Option<&'a str>>, index: u32, name: &'a str) {
    let i = index as usize;
    if i >= names.len() {
        names.resize(i + 1, None);
    }
    if names[i].is_none() {
        names[i] = Some(name);
    }
}

fn name_map_from_dense(names: &[Option<&str>], demangle: bool) -> Option<NameMap> {
    if names.iter().all(Option::is_none) {
        return None;
    }
    let mut map = NameMap::new();
    for (idx, name) in names.iter().enumerate() {
        if let Some(name) = name {
            let name = demangle_symbol_name(name, demangle);
            map.append(idx as u32, &name);
        }
    }
    Some(map)
}

/// Merge `target_features` from linked objects and encode the output custom section.
pub(crate) fn build_target_features_section<'a>(
    features: &[WasmInputTargetFeature<'a>],
    extra_features: &'a [String],
) -> Result<Option<wasm_encoder::CustomSection<'static>>> {
    let mut used: HashSet<&'a str> = HashSet::new();
    let mut disallowed: HashMap<&'a str, crate::input_data::FileId> = HashMap::new();

    for entry in features {
        match entry.feature.prefix {
            TARGET_FEATURE_PREFIX_USED => {
                used.insert(entry.feature.name);
            }
            TARGET_FEATURE_PREFIX_DISALLOWED => {
                disallowed
                    .entry(entry.feature.name)
                    .or_insert(entry.file_id);
            }
            other => {
                bail!(
                    "unrecognized target_features prefix 0x{other:02x} for feature `{}`",
                    entry.feature.name
                );
            }
        }
    }

    for name in &used {
        if let Some(&file_id) = disallowed.get(name) {
            bail!(
                "target feature `{name}` is used by linked objects but disallowed by input file \
                 {file_id}"
            );
        }
    }

    used.extend(extra_features.iter().map(|s| s.as_str()));

    if used.is_empty() {
        return Ok(None);
    }

    let mut feature_names: Vec<&'a str> = used.into_iter().collect();
    feature_names.sort_unstable();

    let mut payload = Vec::new();
    leb128::write::unsigned(&mut payload, feature_names.len() as u64).unwrap();
    for name in feature_names {
        payload.push(TARGET_FEATURE_PREFIX_USED);
        let name_bytes = name.as_bytes();
        leb128::write::unsigned(&mut payload, name_bytes.len() as u64).unwrap();
        payload.extend_from_slice(name_bytes);
    }

    Ok(Some(wasm_encoder::CustomSection {
        name: Cow::Borrowed(TARGET_FEATURES_SECTION_NAME),
        data: Cow::Owned(payload),
    }))
}
