//! Windows host tree.

pub(crate) mod fs {
    use crate::error::Result;
    pub(crate) use crate::host::common::advise_huge_pages_unsupported as advise_huge_pages;
    pub(crate) use crate::host::common::filesystem_kind_unknown as filesystem_kind;
    pub(crate) use crate::host::common::invalidate_mapped_output_noop as invalidate_mapped_output;
    pub(crate) use crate::host::common::preallocate_unsupported as preallocate;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;

    pub(crate) type InputBytes = memmap2::Mmap;

    pub(crate) fn may_have_multiple_links(_metadata: &std::fs::Metadata) -> bool {
        // std::os::windows::fs::MetadataExt::number_of_links is unstable.
        true
    }

    pub(crate) fn read_input(file: &File, path: &Path, prepopulate: bool) -> Result<InputBytes> {
        crate::host::common::map_input(file, path, prepopulate)
    }

    pub(crate) fn release_input_memory(_bytes: &InputBytes) {}

    /// Windows has no execute permission bit; does nothing.
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn make_executable(_file: &File) -> Result {
        Ok(())
    }

    pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
        let path = std::str::from_utf8(bytes).expect("Invalid UTF-8 in archive path name");
        PathBuf::from(path)
    }

    pub(crate) fn create_symlink(target: &Path, dest_path: &Path) -> std::io::Result<()> {
        use std::os::windows::fs::FileTypeExt as _;
        let is_dir = std::fs::metadata(target).is_ok_and(|meta| meta.is_dir());
        let is_symlink_dir =
            std::fs::symlink_metadata(target).is_ok_and(|meta| meta.file_type().is_symlink_dir());
        if is_dir || is_symlink_dir {
            std::os::windows::fs::symlink_dir(target, dest_path)
        } else {
            std::os::windows::fs::symlink_file(target, dest_path)
        }
    }
}

pub(crate) mod os {
    pub(crate) use crate::host::common::kernel_version_unknown as kernel_version;

    #[cfg(test)]
    pub(crate) const SANDBOXED: bool = false;

    pub(crate) const IS_MACOS: bool = false;

    pub(crate) const CLANG_DRIVER_NOOP_SHORT_FLAGS: &[&str] = &[];
}

#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin {
    use crate::error::Result;
    pub(crate) use crate::host::common::UnsupportedPluginLibrary as PluginLibrary;
    use std::ffi::c_int;
    use std::fs::File;

    /// The linker plugin API passes inputs as file descriptors, which Windows doesn't have.
    pub(crate) const SUPPORTED: bool = false;

    pub(crate) type OffT = std::ffi::c_long;

    pub(crate) fn file_descriptor(_file: &File) -> c_int {
        -1
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn increase_file_limit() -> Result {
        Ok(())
    }
}

pub(crate) mod perf {
    pub(crate) use crate::host::common::UnsupportedCounterList as CounterList;
}

pub(crate) mod process {
    pub(crate) use crate::host::common::no_fork::CAN_FORK;
    pub(crate) use crate::host::common::no_fork::ParentNotifier;
    pub(crate) use crate::host::common::no_fork::fork_linker;
}
