use crate::elf::AllowedRange;
use anyhow::Result;

pub mod secnames {
    pub const TEXT_SECTION_NAME_STR: &str = ".text";
    pub const TEXT_SECTION_NAME: &[u8] = TEXT_SECTION_NAME_STR.as_bytes();
    pub const RDATA_SECTION_NAME_STR: &str = ".rdata";
    pub const RDATA_SECTION_NAME: &[u8] = RDATA_SECTION_NAME_STR.as_bytes();
    pub const DATA_SECTION_NAME_STR: &str = ".data";
    pub const DATA_SECTION_NAME: &[u8] = DATA_SECTION_NAME_STR.as_bytes();
    pub const BSS_SECTION_NAME_STR: &str = ".bss";
    pub const BSS_SECTION_NAME: &[u8] = BSS_SECTION_NAME_STR.as_bytes();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationKind {
    /// The absolute address of a symbol or section.
    Absolute,

    /// Like Absolute, but not offset by the image base.
    ImageBaseRelative,

    /// The address of the symbol, relative to the place of the relocation.
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelocationKindInfo {
    pub kind: RelocationKind,
    pub byte_size: u8,
    pub range: AllowedRange,
    pub place_offset: u64,
}

impl RelocationKindInfo {
    #[inline(always)]
    fn verify(&self, value: i64) -> Result<()> {
        anyhow::ensure!(
            self.range.contains(value),
            format!(
                "Relocation {value} outside of bounds [{}, {})",
                self.range.min, self.range.max
            )
        );
        Ok(())
    }

    pub fn write_to_buffer(&self, value: u64, output: &mut [u8]) -> Result<()> {
        self.verify(value as i64)?;

        let byte_size = self.byte_size as usize;
        anyhow::ensure!(
            byte_size <= output.len(),
            "Relocation outside of bounds of section"
        );
        let value_bytes = value.to_le_bytes();
        match byte_size {
            4 => output[..4].copy_from_slice(&value_bytes[..4]),
            8 => output[..8].copy_from_slice(&value_bytes),
            _ => output[..byte_size].copy_from_slice(&value_bytes[..byte_size]),
        }
        Ok(())
    }
}
