//! Types shared between the generic layout and the per-architecture erratum scanners.

use crate::alignment::Alignment;
use crate::output_section_id::OutputSectionId;
use crate::output_section_id::OutputSections;
use crate::output_section_id::SectionLocationInfo;
use crate::output_section_map::OutputSectionMap;
use crate::output_section_part_map::OutputSectionPartMap;
use crate::part_id::PartId;
use crate::platform::Platform;
use object::SectionIndex;
use std::ops::Range;
use strum::IntoEnumIterator as _;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ErratumPatchConfig {
    pub(crate) alignment: Alignment,

    /// Each patch area is rounded up to a multiple of this, so that nothing after it changes page
    /// offset once patches are added.
    pub(crate) size_granularity: u64,
}

/// A patch area appended to every output section that executable input sections map to, so
/// patches stay in reach of the code they're for however a linker script places it.
pub(crate) struct ErratumPatchParts {
    config: ErratumPatchConfig,
    parts: OutputSectionMap<Option<PartId>>,
}

impl ErratumPatchParts {
    pub(crate) fn create<P: Platform>(
        output_sections: &mut OutputSections<P>,
        config: ErratumPatchConfig,
        primaries: &[OutputSectionId],
    ) -> ErratumPatchParts {
        let mut parts = Vec::with_capacity(primaries.len());
        for &primary in primaries {
            // An empty location counter range, since the primary already emitted its own.
            let location_info = output_sections
                .output_info(primary)
                .location_info
                .as_ref()
                .map(|info| SectionLocationInfo {
                    location_counters: (info.location_counters.1, info.location_counters.1),
                    location: None,
                    at_location: None,
                    is_top_level: false,
                });

            let section_id = output_sections.add_secondary_section(
                primary,
                config.alignment,
                None,
                location_info,
            );

            parts.push((
                primary,
                section_id.part_id_with_alignment::<P>(config.alignment),
            ));
        }

        let mut map = output_sections.new_section_map();
        for (primary, part_id) in parts {
            *map.get_mut(primary) = Some(part_id);
        }

        ErratumPatchParts { config, parts: map }
    }

    pub(crate) fn config(&self) -> ErratumPatchConfig {
        self.config
    }

    pub(crate) fn for_part<P: Platform>(
        &self,
        output_sections: &OutputSections<P>,
        part_id: PartId,
    ) -> Option<PartId> {
        let primary = output_sections.primary_output_section(part_id.output_section_id::<P>());
        *self.parts.get(primary)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::EnumIter)]
pub(crate) enum Erratum {
    /// A memory access immediately followed by a non-SIMD integer multiply-accumulate writing a
    /// 64-bit register.
    Mac835769,

    /// An `ADRP` writing `Xn` at page offset `0xff8` or `0xffc`, followed within one or two
    /// instructions by a load or store using `Xn` as its base register.
    Adrp843419,
}

impl Erratum {
    /// The moved instruction, the branch back, and for 835769 specifically, the `NOP` that ends up
    /// between the memory access and the multiply-accumulate.
    pub(crate) const fn patch_size(self) -> u64 {
        match self {
            Erratum::Mac835769 => 12,
            Erratum::Adrp843419 => 8,
        }
    }

    const fn flag(self) -> u8 {
        match self {
            Erratum::Mac835769 => 1,
            Erratum::Adrp843419 => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ErratumFixes(u8);

impl ErratumFixes {
    pub(crate) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn insert(&mut self, erratum: Erratum) {
        self.0 |= erratum.flag();
    }

    pub(crate) const fn remove(&mut self, erratum: Erratum) {
        self.0 &= !erratum.flag();
    }
}

impl IntoIterator for ErratumFixes {
    type Item = Erratum;
    type IntoIter = ErratumFixesIter;

    fn into_iter(self) -> ErratumFixesIter {
        ErratumFixesIter(self.0)
    }
}

pub(crate) struct ErratumFixesIter(u8);

impl Iterator for ErratumFixesIter {
    type Item = Erratum;

    fn next(&mut self) -> Option<Erratum> {
        let erratum = Erratum::iter().find(|erratum| self.0 & erratum.flag() != 0)?;

        self.0 &= !erratum.flag();

        Some(erratum)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PatchSite {
    pub(crate) section_index: SectionIndex,

    /// Of the instruction that gets replaced with a branch, within its input section.
    pub(crate) offset: u64,

    pub(crate) erratum: Erratum,

    /// Of this entry within its [`PatchArea`]. Assigned by [`ObjectPatches::new`].
    patch_offset: u64,
}

impl PatchSite {
    pub(crate) fn new(section_index: SectionIndex, offset: u64, erratum: Erratum) -> PatchSite {
        PatchSite {
            section_index,
            offset,
            erratum,
            patch_offset: 0,
        }
    }

    pub(crate) fn patch_offset(self) -> u64 {
        self.patch_offset
    }

    pub(crate) fn patch_range(self) -> Range<usize> {
        let start = self.patch_offset as usize;
        start..start + self.erratum.patch_size() as usize
    }
}

/// One object's slice of a patch part.
#[derive(Debug)]
pub(crate) struct PatchArea {
    pub(crate) part_id: PartId,
    sites: Range<usize>,
    size: u64,
    padding: u64,
    base: u64,
}

impl PatchArea {
    pub(crate) fn size(&self) -> u64 {
        self.size + self.padding
    }

    pub(crate) fn padding(&self) -> u64 {
        self.padding
    }

    /// Only the last object with patches in the part gets any, to pad the part up to
    /// [`ErratumPatchConfig::size_granularity`].
    pub(crate) fn set_padding(&mut self, padding: u64) {
        self.padding = padding;
    }

    /// Where this area starts, once the part has an address.
    pub(crate) fn base(&self) -> u64 {
        self.base
    }
}

/// Every instruction in one object that needs patching, each with its slot in the object's patch
/// areas already assigned.
#[derive(Debug, Default)]
pub(crate) struct ObjectPatches {
    /// Grouped by area, then sorted by `(section_index, offset)`.
    sites: Vec<PatchSite>,

    areas: Vec<PatchArea>,
}

impl ObjectPatches {
    pub(crate) fn new(areas: Vec<(PartId, Vec<PatchSite>)>) -> ObjectPatches {
        let mut sites = Vec::new();
        let mut out_areas = Vec::new();

        for (part_id, mut area_sites) in areas {
            if area_sites.is_empty() {
                continue;
            }

            area_sites.sort_unstable_by_key(|site| (site.section_index.0, site.offset));

            let mut size = 0;
            for site in &mut area_sites {
                site.patch_offset = size;
                size += site.erratum.patch_size();
            }

            let start = sites.len();
            sites.extend(area_sites);

            out_areas.push(PatchArea {
                part_id,
                sites: start..sites.len(),
                size,
                padding: 0,
                base: 0,
            });
        }

        ObjectPatches {
            sites,
            areas: out_areas,
        }
    }

    pub(crate) fn areas(&self) -> &[PatchArea] {
        &self.areas
    }

    pub(crate) fn area_mut(&mut self, part_id: PartId) -> Option<&mut PatchArea> {
        self.areas.iter_mut().find(|area| area.part_id == part_id)
    }

    pub(crate) fn assign_bases(&mut self, memory_offsets: &mut OutputSectionPartMap<u64>) {
        for area in &mut self.areas {
            let addr = memory_offsets.get_mut(area.part_id);
            area.base = *addr;
            *addr += area.size();
        }
    }

    /// The area holding `section_index`'s patches, with its index into [`areas`](Self::areas).
    pub(crate) fn in_section(&self, section_index: SectionIndex) -> Option<(usize, &[PatchSite])> {
        self.areas.iter().enumerate().find_map(|(index, area)| {
            let sites = &self.sites[area.sites.clone()];
            let start = sites.partition_point(|site| site.section_index.0 < section_index.0);
            let end = sites.partition_point(|site| site.section_index.0 <= section_index.0);

            (start < end).then(|| (index, &sites[start..end]))
        })
    }
}
