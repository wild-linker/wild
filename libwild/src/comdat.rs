use crate::error::Result;
use crate::hash::PassThroughHashMap;
use crate::hash::PreHashed;
use crate::input_data::FileId;
use crate::input_section_id::InputSectionIndex;
use crate::platform::Args as _;
use crate::platform::ObjectFile as _;
use crate::platform::Platform;
use crate::resolution::ResolvedFile;
use crate::resolution::ResolvedGroup;
use crate::resolution::SectionSlot;
use crate::symbol_db::SymbolDb;
use crossbeam_queue::SegQueue;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::num::NonZeroU32;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub(crate) struct SectionGroup<'data> {
    /// The index of the section that defines the section group.
    pub(crate) index: InputSectionIndex,

    /// The symbol that identifies the section group.
    pub(crate) signature_symbol: object::SymbolIndex,

    pub(crate) is_comdat: bool,

    /// The section indices of the sections within the section group.
    pub(crate) member_data: &'data [object::endian::U32<object::LittleEndian>],
}

impl SectionGroup<'_> {
    pub(crate) fn members(&self) -> impl ExactSizeIterator<Item = object::SectionIndex> + '_ {
        self.member_data
            .iter()
            .map(|member| object::SectionIndex(member.get(object::LittleEndian) as usize))
    }
}

#[derive(Debug, Default)]
pub(crate) struct ObjectSectionGroups<'data> {
    pub(crate) groups: Vec<SectionGroup<'data>>,

    /// Index into the shared selection array in `Selection::select` for the start of the section
    /// groups for this object.
    selection_start: usize,

    /// Indexed as for sections in the object. Indicates which sections have been discarded due to
    /// being part of a COMDAT group that lost to another COMDAT group with the same signature.
    /// Note, this can't currently be a separate SectionSlot state, since we decide which such
    /// sections to discard before section resolution.
    discarded: Vec<bool>,

    /// Indexed as for sections in the object. Contains all memberships until `prepare_gc`, then
    /// only groups that still need GC propagation. `visit_group_once` consumes the remaining
    /// entries.
    membership: Vec<Option<NonZeroU32>>,
}

impl<'data> ObjectSectionGroups<'data> {
    /// Only valid before `prepare_gc`, while the membership map is complete.
    pub(crate) fn group_for_section_before_gc(
        &self,
        index: object::SectionIndex,
    ) -> Option<&SectionGroup<'data>> {
        self.pending_group_index(index).map(|id| &self.groups[id])
    }

    fn pending_group_index(&self, index: object::SectionIndex) -> Option<usize> {
        self.membership
            .get(index.0)
            .copied()
            .flatten()
            .map(|id| (id.get() - 1) as usize)
    }

    /// Remove groups with only a single GC-eligible member from GC consideration. This eliminates a
    /// bit of unnecessary work during the GC phase.
    pub(crate) fn prepare_gc(&mut self, sections: &[SectionSlot], gc_enabled: bool) {
        let data = self;
        if !gc_enabled {
            data.membership = Vec::new();
            return;
        }

        let mut needs_membership = false;

        for group in &data.groups {
            let members = || group.members().chain(std::iter::once(group.index.into()));

            let participating = members()
                .filter(|member| {
                    !matches!(
                        sections[member.0],
                        SectionSlot::Discard | SectionSlot::MergeStrings(_)
                    )
                })
                .take(2)
                .count();

            if participating < 2 {
                for member in members() {
                    data.membership[member.0] = None;
                }
            } else {
                needs_membership = true;
            }
        }

        if !needs_membership {
            data.membership = Vec::new();
        }
    }

    /// Consumes the group's pending membership and visits its members and header. Later requests
    /// through any member of the same group do nothing.
    pub(crate) fn visit_group_once(
        &mut self,
        index: object::SectionIndex,
        mut visit: impl FnMut(object::SectionIndex),
    ) {
        let Some(id) = self.pending_group_index(index) else {
            return;
        };

        let section_group = &self.groups[id];

        for member in section_group
            .members()
            .chain(std::iter::once(section_group.index.into()))
        {
            self.membership[member.0] = None;
            visit(member);
        }
    }

    pub(crate) fn release_gc_phase_resources(&mut self) {
        self.membership = Vec::new();
    }

    pub(crate) fn is_discarded(&self, index: object::SectionIndex) -> bool {
        self.discarded.get(index.0).copied().unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SectionGroupId {
    file_id: FileId,

    /// Index in the global selection array: the object's selection_start plus its group index.
    selection_index: u32,
}

type WinnerMap<'data> = PassThroughHashMap<&'data [u8], SectionGroupId>;

type Candidate<'data> = (PreHashed<&'data [u8]>, SectionGroupId);

#[derive(Default)]
pub(crate) struct Selection<'data> {
    num_groups: usize,
    candidates: Vec<Vec<Vec<Candidate<'data>>>>,
    ir_claims: PassThroughHashMap<&'data [u8], FileId>,

    /// Index of the first input group added by the linker plugin. Before this is set, all native
    /// objects are original inputs for COMDAT selection.
    first_lto_output_group: Option<usize>,
}

impl<'data> Selection<'data> {
    #[cfg(feature = "plugins")]
    pub(crate) fn begin_lto_output(&mut self, first_group: usize) {
        self.first_lto_output_group = Some(first_group);
    }

    pub(crate) fn select<P: Platform>(
        &mut self,
        resolved: &mut [ResolvedGroup<'data, P>],
        inputs: Vec<(FileId, Vec<SectionGroup<'data>>)>,
        symbol_db: &SymbolDb<'data, P>,
    ) -> Result {
        if inputs.is_empty()
            && self.candidates.iter().all(Vec::is_empty)
            && symbol_db.num_lto_objects() == 0
        {
            return Ok(());
        }

        crate::timing_phase!("Select COMDAT groups");

        let new_objects = self.assign_object_groups(resolved, inputs);

        let candidates = Self::bucket_candidates(
            resolved,
            new_objects,
            symbol_db.args.common().available_threads.get(),
        );

        self.add_candidates(candidates);

        let selected = (0..self.num_groups)
            .map(|_| AtomicBool::new(false))
            .collect::<Vec<_>>();

        let winners = self.select_native_groups(symbol_db, &selected);

        #[cfg(feature = "plugins")]
        self.select_ir_groups(resolved, symbol_db, &winners);

        Self::apply_selection(resolved, &selected)?;

        {
            crate::timing_phase!("Drop COMDAT winner maps");
            winners.into_par_iter().for_each(|winners| {
                crate::verbose_timing_phase!("Drop COMDAT bucket winner map");
                drop(winners);
            });
        }
        Ok(())
    }

    fn assign_object_groups<P: Platform>(
        &mut self,
        resolved: &mut [ResolvedGroup<'data, P>],
        inputs: Vec<(FileId, Vec<SectionGroup<'data>>)>,
    ) -> Vec<Vec<FileId>> {
        crate::timing_phase!("Assign COMDAT metadata to objects");
        let mut new_objects = vec![Vec::new(); resolved.len()];

        for (file_id, section_groups) in inputs {
            let ResolvedFile::Object(object) = &mut resolved[file_id.group()].files[file_id.file()]
            else {
                continue;
            };

            object.section_groups.selection_start = self.num_groups;
            self.num_groups += section_groups.len();
            object.section_groups.groups = section_groups;
            new_objects[file_id.group()].push(file_id);
        }

        new_objects
    }

    fn bucket_candidates<P: Platform>(
        resolved: &[ResolvedGroup<'data, P>],
        new_objects: Vec<Vec<FileId>>,
        num_buckets: usize,
    ) -> Vec<Vec<Vec<Candidate<'data>>>> {
        crate::timing_phase!("Bucket COMDAT candidates");
        let outputs = (0..num_buckets)
            .map(|_| SegQueue::new())
            .collect::<Vec<_>>();

        new_objects.into_par_iter().for_each(|file_ids| {
            if file_ids.is_empty() {
                return;
            }

            crate::verbose_timing_phase!("Bucket COMDAT candidates for input group");

            let mut buckets = vec![Vec::new(); num_buckets];

            for file_id in file_ids {
                let ResolvedFile::Object(object) = &resolved[file_id.group()].files[file_id.file()]
                else {
                    unreachable!();
                };

                for (index, section_group) in object.section_groups.groups.iter().enumerate() {
                    if section_group.is_comdat {
                        let symbol = object
                            .common
                            .object
                            .symbol(section_group.signature_symbol)
                            .expect("Invalid group signature symbol");
                        let name = object
                            .common
                            .object
                            .symbol_name(symbol)
                            .expect("Invalid group signature name");
                        let signature = PreHashed::new(name, crate::hash::hash_bytes(name));
                        buckets[signature.hash() as usize % num_buckets].push((
                            signature,
                            SectionGroupId {
                                file_id,
                                selection_index: u32::try_from(
                                    object.section_groups.selection_start + index,
                                )
                                .expect("COMDAT groups overflowed 32 bits"),
                            },
                        ));
                    }
                }
            }

            for (output, bucket) in outputs.iter().zip(buckets) {
                if !bucket.is_empty() {
                    output.push(bucket);
                }
            }
        });

        outputs
            .into_par_iter()
            .map(|queue| queue.into_iter().collect())
            .collect()
    }

    fn add_candidates(&mut self, candidates: Vec<Vec<Vec<Candidate<'data>>>>) {
        crate::timing_phase!("Accumulate COMDAT candidates");

        let num_buckets = candidates.len();
        self.candidates.resize_with(num_buckets, Vec::new);

        self.candidates
            .par_iter_mut()
            .zip(candidates.into_par_iter())
            .for_each(|(existing, new)| {
                crate::verbose_timing_phase!("Accumulate COMDAT bucket candidates");

                if existing.is_empty() {
                    *existing = new;
                } else {
                    existing.extend(new);
                }
            });
    }

    /// Select which native (as in non-linker-plugin) groups win.
    fn select_native_groups<P: Platform>(
        &mut self,
        symbol_db: &SymbolDb<'data, P>,
        selected: &[AtomicBool],
    ) -> Vec<WinnerMap<'data>> {
        crate::timing_phase!("Select native COMDAT winners");

        let retain_candidates = symbol_db.num_lto_objects() > 0;

        self.candidates
            .par_iter_mut()
            .map(|bucket| {
                crate::verbose_timing_phase!("Select COMDAT bucket winners");

                let mut winners: WinnerMap<'data> = Default::default();

                for &(signature, candidate) in bucket.iter().flatten() {
                    // Once the plugin has been told that IR prevails, a native archive member
                    // extracted during code generation must not displace it. The plugin's native
                    // output can supply the replacement definition for that IR group.
                    if self.ir_claims.contains_key(&signature)
                        && self
                            .first_lto_output_group
                            .is_none_or(|first| candidate.file_id.group() < first)
                    {
                        continue;
                    }

                    winners
                        .entry(signature)
                        .and_modify(|winner| {
                            *winner = std::cmp::min_by_key(*winner, candidate, |id| {
                                (id.file_id, id.selection_index)
                            });
                        })
                        .or_insert(candidate);
                }

                // These atomic writes shouldn't overlap, since each store should be to a different
                // slot. We only use atomics because it isn't practical to give each worker
                // exclusive access to the parts that it needs.
                for winner in winners.values() {
                    selected[winner.selection_index as usize].store(true, Ordering::Relaxed);
                }

                if !retain_candidates {
                    crate::verbose_timing_phase!("Drop COMDAT bucket candidates");
                    bucket.clear();
                }

                winners
            })
            .collect()
    }

    #[cfg(feature = "plugins")]
    fn select_ir_groups<P: Platform>(
        &mut self,
        resolved: &mut [ResolvedGroup<'data, P>],
        symbol_db: &SymbolDb<'data, P>,
        winners: &[WinnerMap<'data>],
    ) {
        crate::timing_phase!("Select IR COMDAT winners");
        let num_buckets = winners.len();

        for input_group in resolved.iter_mut() {
            for file in &mut input_group.files {
                let ResolvedFile::LtoInput(input) = file else {
                    continue;
                };

                let crate::grouping::SequencedInput::LtoInput(object) =
                    symbol_db.file(input.file_id)
                else {
                    unreachable!()
                };

                input.comdat_discarded.resize(object.symbols.len(), false);
                for (index, symbol) in object.symbols.iter().enumerate() {
                    if let Some(signature) = symbol.comdat_key {
                        let native = winners[signature.hash() as usize % num_buckets]
                            .contains_key(&signature);
                        let owner = if let Some(owner) = self.ir_claims.get(&signature) {
                            Some(*owner)
                        } else if !native {
                            self.ir_claims.insert(signature, input.file_id);
                            Some(input.file_id)
                        } else {
                            None
                        };
                        input.comdat_discarded[index] = owner != Some(input.file_id);
                    }
                }
            }
        }
    }

    fn apply_selection<P: Platform>(
        resolved: &mut [ResolvedGroup<'data, P>],
        selected: &[AtomicBool],
    ) -> Result {
        crate::timing_phase!("Apply COMDAT selection");

        resolved
            .par_iter_mut()
            .try_for_each(|input_group| -> Result {
                crate::verbose_timing_phase!("Apply COMDAT selection to input group");

                for file in &mut input_group.files {
                    let ResolvedFile::Object(object) = file else {
                        continue;
                    };

                    let section_groups = &mut object.section_groups;

                    if section_groups.groups.is_empty() {
                        continue;
                    }

                    {
                        crate::verbose_timing_phase!("Allocate COMDAT section maps");
                        section_groups.discarded = vec![false; object.section_id_range.len()];
                        section_groups.membership = vec![None; object.section_id_range.len()];
                    }

                    crate::verbose_timing_phase!("Mark COMDAT sections");

                    for (id, section_group) in section_groups.groups.iter().enumerate() {
                        let keep = !section_group.is_comdat
                            || selected[section_groups.selection_start + id]
                                .load(Ordering::Relaxed);
                        let id = NonZeroU32::new(id as u32 + 1).unwrap();

                        section_groups.discarded[section_group.index.as_usize()] = !keep;

                        for member in section_group.members() {
                            if section_groups.membership[member.0].replace(id).is_some() {
                                crate::bail!(
                                    "{}: section {} belongs to multiple groups",
                                    object.common.input,
                                    member.0
                                );
                            }

                            section_groups.discarded[member.0] = !keep;
                        }
                    }
                }

                Ok(())
            })
    }
}
