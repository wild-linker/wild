pub mod secnames {
    pub const TEXT_SECTION_NAME_STR: &str = ".text";
    pub const TEXT_SECTION_NAME: &[u8] = TEXT_SECTION_NAME_STR.as_bytes();
    pub const RDATA_SECTION_NAME_STR: &str = ".rodata";
    pub const RDATA_SECTION_NAME: &[u8] = RDATA_SECTION_NAME_STR.as_bytes();
    pub const DATA_SECTION_NAME_STR: &str = ".data";
    pub const DATA_SECTION_NAME: &[u8] = DATA_SECTION_NAME_STR.as_bytes();
    pub const BSS_SECTION_NAME_STR: &str = ".bss";
    pub const BSS_SECTION_NAME: &[u8] = BSS_SECTION_NAME_STR.as_bytes();
}
