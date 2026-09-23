//! WASI host tree.

pub(crate) mod fs {
    use crate::error::Context as _;
    use crate::error::Result;
    pub(crate) use crate::host::common::advise_huge_pages_unsupported as advise_huge_pages;
    pub(crate) use crate::host::common::filesystem_kind_unknown as filesystem_kind;
    pub(crate) use crate::host::common::invalidate_mapped_output_noop as invalidate_mapped_output;
    pub(crate) use crate::host::common::preallocate_unsupported as preallocate;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;

    /// WASI can't map files, so inputs are read into memory.
    pub(crate) struct InputBytes(Vec<u8>);

    impl std::fmt::Debug for InputBytes {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("FileBytes").finish_non_exhaustive()
        }
    }

    impl std::ops::Deref for InputBytes {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub(crate) fn may_have_multiple_links(_metadata: &std::fs::Metadata) -> bool {
        // std::os::wasi::fs::MetadataExt is unstable
        true
    }

    pub(crate) fn read_input(file: &File, path: &Path, _prepopulate: bool) -> Result<InputBytes> {
        use std::io::Read as _;
        let mut bytes = Vec::new();
        let mut file = file;
        file.read_to_end(&mut bytes)
            .with_context(|| format!("Failed to read file `{}`", path.display()))?;
        Ok(InputBytes(bytes))
    }

    pub(crate) fn release_input_memory(_bytes: &InputBytes) {}

    /// WASI has no execute permission bit; does nothing.
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn make_executable(_file: &File) -> Result {
        Ok(())
    }

    pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
        use std::ffi::OsStr;
        use std::os::wasi::ffi::OsStrExt as _;
        std::path::Path::new(OsStr::from_bytes(bytes)).to_path_buf()
    }

    pub(crate) fn create_symlink(_target: &Path, _dest_path: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "creating symlinks on wasi not supported on stable rust",
        ))
    }
}

pub(crate) mod os {
    pub(crate) use crate::host::common::kernel_version_unknown as kernel_version;

    /// WASI only sees preopened directories, and has no temporary directory or subprocesses.
    #[cfg(test)]
    pub(crate) const SANDBOXED: bool = true;

    pub(crate) const IS_MACOS: bool = false;

    pub(crate) const CLANG_DRIVER_NOOP_SHORT_FLAGS: &[&str] = &[];
}

#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin {
    use crate::error::Result;
    pub(crate) use crate::host::common::UnsupportedPluginLibrary as PluginLibrary;
    use std::ffi::c_int;
    use std::fs::File;

    /// WASI can't load dynamic libraries.
    pub(crate) const SUPPORTED: bool = false;

    pub(crate) type OffT = i64;

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
