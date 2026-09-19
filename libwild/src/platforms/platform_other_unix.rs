//! Host tree for unix systems without a dedicated tree (e.g. the BSDs).

pub(crate) mod fs {
    pub(crate) use crate::platforms::common::advise_huge_pages_unsupported as advise_huge_pages;
    pub(crate) use crate::platforms::common::filesystem_kind_unknown as filesystem_kind;
    pub(crate) use crate::platforms::common::invalidate_mapped_output_noop as invalidate_mapped_output;
    pub(crate) use crate::platforms::common::preallocate_unsupported as preallocate;
    pub(crate) use crate::platforms::platform_unix::fs::InputBytes;
    pub(crate) use crate::platforms::platform_unix::fs::create_symlink;
    pub(crate) use crate::platforms::platform_unix::fs::make_executable;
    pub(crate) use crate::platforms::platform_unix::fs::path_from_bytes;
    pub(crate) use crate::platforms::platform_unix::fs::read_input;
    pub(crate) use crate::platforms::platform_unix::fs::release_input_memory;
}

pub(crate) mod host {
    pub(crate) use crate::platforms::common::kernel_version_unknown as kernel_version;
    #[cfg(test)]
    pub(crate) use crate::platforms::platform_unix::host::SANDBOXED;

    pub(crate) const IS_MACOS: bool = false;

    pub(crate) const CLANG_DRIVER_NOOP_SHORT_FLAGS: &[&str] = &[];
}

#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin {
    pub(crate) use crate::platforms::platform_unix::linker_plugin::OffT;
    pub(crate) use crate::platforms::platform_unix::linker_plugin::PluginLibrary;
    pub(crate) use crate::platforms::platform_unix::linker_plugin::SUPPORTED;
    pub(crate) use crate::platforms::platform_unix::linker_plugin::file_descriptor;
    pub(crate) use crate::platforms::platform_unix::linker_plugin::increase_file_limit;
}

pub(crate) mod perf {
    pub(crate) use crate::platforms::common::UnsupportedCounterList as CounterList;
}

pub(crate) mod process {
    pub(crate) use crate::platforms::platform_unix::process::CAN_FORK;
    pub(crate) use crate::platforms::platform_unix::process::ParentNotifier;
    pub(crate) use crate::platforms::platform_unix::process::fork_linker;
}
