//! Host filesystem services: reading inputs, output preallocation and page advice, file
//! permissions, symlinks and path conversion.

pub(crate) use super::imp::fs::InputBytes;
pub(crate) use super::imp::fs::advise_huge_pages;
pub(crate) use super::imp::fs::create_symlink;
pub(crate) use super::imp::fs::filesystem_kind;
pub(crate) use super::imp::fs::invalidate_mapped_output;
pub(crate) use super::imp::fs::make_executable;
pub(crate) use super::imp::fs::may_have_multiple_links;
pub(crate) use super::imp::fs::path_from_bytes;
pub(crate) use super::imp::fs::preallocate;
pub(crate) use super::imp::fs::read_input;
pub(crate) use super::imp::fs::release_input_memory;

/// The kinds of filesystem that change wild's default output strategy.
// Only constructed on hosts whose tree detects filesystems.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilesystemKind {
    Btrfs,
    Vfat,
    /// ext2, ext3 or ext4. They share a superblock magic.
    Ext4,
    Xfs,
    Other,
}
