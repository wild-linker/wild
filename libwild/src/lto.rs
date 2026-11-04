use crate::elf;
use crate::elf::Elf;
use crate::elf::ElfClass;
use crate::error::Result;
use crate::input_data::FileId;
use crate::input_data::InputRef;
use crate::platform::Args as _;
use crate::platform::Platform;
use crate::resolution::ResolutionResources;
use crate::resolution::ResolvedFile;
use crate::resolution::ResolvedGroup;
use crate::resolution::SymbolAttributes;
use crate::symbol::UnversionedSymbolName;
use crate::symbol_db::SymbolDb;
use crate::symbol_db::SymbolId;
use crate::symbol_db::SymbolIdRange;
use crate::value_flags::FlagsForSymbol;
use crate::value_flags::PerSymbolFlags;
use crate::value_flags::ValueFlags;
use crossbeam_utils::atomic::AtomicCell;
use rayon::Scope;

pub(crate) struct SymbolPropertiesDisplay<'data>(&'data LtoSymbol<'data>);
#[derive(Debug)]
pub(crate) struct LtoInput<'data> {
    pub(crate) file_id: FileId,
    pub(crate) symbol_id_range: SymbolIdRange,
    pub(crate) section_id_range: crate::input_section_id::SectionIdRange,
    pub(crate) input_ref: InputRef<'data>,
    pub(crate) symbols: Vec<LtoSymbol<'data>>,
    /// Set to false once symbols from this object should be ignored. This is done once LTO has
    /// been performed.
    pub(crate) enabled: bool,
}

#[derive(Debug)]
pub(crate) struct LtoInputInfo<'data> {
    pub(crate) input_ref: InputRef<'data>,
    pub(crate) symbols: Vec<LtoSymbol<'data>>,
    pub(crate) handle: Option<&'data AtomicCell<Option<FileId>>>,
}

impl<'data> LtoInputInfo<'data> {
    pub(crate) fn num_symbols(&self) -> usize {
        self.symbols.len()
    }

    pub(crate) fn into_input_object(
        self,
        file_id: FileId,
        symbol_id_range: SymbolIdRange,
    ) -> LtoInput<'data> {
        if let Some(handle) = self.handle {
            handle.store(Some(file_id));
        }

        LtoInput {
            file_id,
            symbol_id_range,
            section_id_range: crate::input_section_id::SectionIdRange::empty(),
            input_ref: self.input_ref,
            symbols: self.symbols,
            enabled: true,
        }
    }
}

impl<'data> LtoInput<'data> {
    pub(crate) fn symbol_name(
        &self,
        symbol_id: crate::symbol_db::SymbolId,
    ) -> UnversionedSymbolName<'data> {
        let local_index = self.symbol_id_range.id_to_offset(symbol_id);
        self.symbols[local_index].name
    }

    pub(crate) fn symbol_visibility(
        &self,
        symbol_id: crate::symbol_db::SymbolId,
    ) -> crate::symbol_db::Visibility {
        let local_index = self.symbol_id_range.id_to_offset(symbol_id);
        crate::elf::convert_elf_visibility(object::elf::SymbolVisibility(
            self.symbols[local_index].visibility,
        ))
    }

    pub(crate) fn symbols_iter(&self) -> impl Iterator<Item = (SymbolId, &LtoSymbol<'data>)> {
        self.symbol_id_range.into_iter().zip(self.symbols.iter())
    }

    pub(crate) fn symbol_properties_display(
        &'_ self,
        symbol_id: SymbolId,
    ) -> SymbolPropertiesDisplay<'_> {
        SymbolPropertiesDisplay(&self.symbols[self.symbol_id_range.id_to_offset(symbol_id)])
    }

    pub(crate) fn is_optional(&self) -> bool {
        self.input_ref.has_archive_semantics() && !self.input_ref.file.modifiers.whole_archive
    }
}

#[derive(Debug)]
pub(crate) struct LtoSymbol<'data> {
    pub(crate) name: UnversionedSymbolName<'data>,
    pub(crate) version: Option<&'data [u8]>,
    pub(crate) comdat_key: Option<crate::hash::PreHashed<&'data [u8]>>,
    pub(crate) visibility: u8,
    pub(crate) kind: Option<SymbolKind>,
    pub(crate) size: u64,
}

impl LtoSymbol<'_> {
    pub(crate) fn is_definition(&self) -> bool {
        self.kind.is_some_and(|kind| kind.is_definition())
    }
}

impl std::fmt::Display for SymbolPropertiesDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LTO ")?;
        if let Some(kind) = self.0.kind {
            write!(f, "{kind:?}")?;
        } else {
            write!(f, "UNKNOWN")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for LtoInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LTO input `{}`", self.input_ref)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Def = 0,
    WeakDef = 1,
    Undef = 2,
    WeakUndef = 3,
    Common = 4,
}

impl SymbolKind {
    pub(crate) fn is_definition(self) -> bool {
        matches!(
            self,
            SymbolKind::Def | SymbolKind::WeakDef | SymbolKind::Common
        )
    }
}

pub(crate) fn resolve_lto_symbols<'data, 'scope, C: ElfClass>(
    obj: &crate::lto::LtoInput<'data>,
    resources: &'scope ResolutionResources<'data, 'scope, Elf<C>>,
    definitions_out: &mut [SymbolId],
    scope: &Scope<'scope>,
) -> Result {
    obj.symbols
        .iter()
        .enumerate()
        .zip(definitions_out)
        .try_for_each(
            |((local_symbol_index, local_symbol), definition)| -> Result {
                if !local_symbol.is_definition() {
                    let mut name_info = Elf::<C>::parse_raw_symbol_name(local_symbol.name.bytes());
                    if let Some(version) = local_symbol.version {
                        name_info.version_name = Some(version);
                    }

                    let symbol_attributes = SymbolAttributes {
                        name_info,
                        is_local: false,
                        default_visibility: local_symbol.visibility == object::elf::STV_DEFAULT.0,
                        is_weak: local_symbol.kind == Some(crate::lto::SymbolKind::WeakUndef),
                    };

                    crate::resolution::resolve_symbol(
                        obj.symbol_id_range.offset_to_id(local_symbol_index),
                        &symbol_attributes,
                        definition,
                        resources,
                        false,
                        obj.file_id,
                        scope,
                        true,
                    )?;
                }

                Ok(())
            },
        )
}

pub(crate) fn mark_wrap_symbols_as_non_ir_ref<'data, P: Platform>(
    symbol_db: &SymbolDb<'data, P>,
    per_symbol_flags: &mut PerSymbolFlags,
) {
    for name in symbol_db.args.symbol_names_to_wrap() {
        for lookup_name in [
            name.clone(),
            format!("__wrap_{name}"),
            format!("__real_{name}"),
        ] {
            if let Some(symbol_id) =
                symbol_db.get_unversioned(&UnversionedSymbolName::prehashed(lookup_name.as_bytes()))
            {
                per_symbol_flags.set_flag(symbol_id, ValueFlags::HAS_NON_IR_REF);
            }
        }
    }
}

/// Marks the symbols named by -init and -fini as having non-IR references. Only DT_INIT and
/// DT_FINI refer to them, so without this the plugin could internalise or remove them.
pub(crate) fn mark_init_fini_symbols_as_non_ir_ref<'data, C: ElfClass>(
    symbol_db: &SymbolDb<'data, Elf<C>>,
    per_symbol_flags: &mut PerSymbolFlags,
) {
    for name in [&symbol_db.args.init, &symbol_db.args.fini]
        .into_iter()
        .flatten()
    {
        if let Some(symbol_id) =
            symbol_db.get_unversioned(&UnversionedSymbolName::prehashed(name.as_bytes()))
        {
            // Mark the selected definition, which isn't necessarily the first symbol with this
            // name, e.g. a weak definition may precede the strong one.
            per_symbol_flags.set_flag(symbol_db.definition(symbol_id), ValueFlags::HAS_NON_IR_REF);
        }
    }
}

pub(crate) fn mark_lto_symbols_for_dynamic_export<C: ElfClass>(
    symbol_db: &SymbolDb<Elf<C>>,
    per_symbol_flags: &mut PerSymbolFlags,
    resolved_groups: &[ResolvedGroup<Elf<C>>],
) {
    use crate::grouping::Group;

    for group in resolved_groups {
        for file in &group.files {
            if let ResolvedFile::LtoInput(lto_input) = file {
                let Group::LtoInputs(files) = &symbol_db.groups[lto_input.file_id.group()] else {
                    unreachable!();
                };
                let file = &files[lto_input.file_id.file()];

                let Some(mode) = crate::layout::export_symbols_mode(symbol_db, &file.input_ref)
                else {
                    continue;
                };

                for (symbol_id, symbol) in file.symbols_iter() {
                    if symbol.is_definition()
                        && crate::layout::can_export_global_def(
                            symbol_db,
                            elf::convert_elf_visibility(object::elf::SymbolVisibility(
                                symbol.visibility,
                            )),
                            symbol_id,
                            per_symbol_flags.flags_for_symbol(symbol_id),
                            mode,
                        )
                    {
                        per_symbol_flags.set_flag(symbol_id, ValueFlags::EXPORT_DYNAMIC);
                    }
                }
            }
        }
    }
}

pub(crate) fn has_loaded_lto_input<P: Platform>(resolved_groups: &[ResolvedGroup<P>]) -> bool {
    resolved_groups.iter().any(|group| {
        group
            .files
            .iter()
            .any(|file| matches!(file, ResolvedFile::LtoInput(_)))
    })
}

pub(crate) fn ingest_generated_inputs<'data, C: ElfClass>(
    plugin_loaded: crate::input_data::LoadedInputs<'data, Elf<C>>,
    symbol_db: &mut SymbolDb<'data, Elf<C>>,
    resolver: &mut crate::resolution::Resolver<'data, Elf<C>>,
    per_symbol_flags: &mut PerSymbolFlags,
    output_sections: &mut crate::output_section_id::OutputSections<'data, Elf<C>>,
    layout_rules_builder: &mut crate::layout_rules::LayoutRulesBuilder<'data>,
) -> Result {
    // Temporarily restore original (pre-wrap) name mappings so that definitions of wrapped
    // names in the LTO output are registered as alternatives of the original symbols rather
    // than of `__wrap_*`.
    symbol_db.restore_wrapped_symbol_names();

    resolver.begin_lto_output(symbol_db.groups.len());
    symbol_db.add_inputs(
        per_symbol_flags,
        output_sections,
        layout_rules_builder,
        plugin_loaded,
    )?;

    // Re-apply --wrap overrides so that wrapped names now map to definitions from the LTO
    // output rather than the LTO input objects.
    symbol_db.apply_wrapped_symbol_overrides();

    resolver.resolve_symbols_and_select_archive_entries(symbol_db, per_symbol_flags)?;

    symbol_db.disable_lto_inputs();

    crate::symbol_db::resolve_alternative_symbol_definitions(
        symbol_db,
        per_symbol_flags,
        &resolver.resolved_groups,
    )?;

    Ok(())
}
