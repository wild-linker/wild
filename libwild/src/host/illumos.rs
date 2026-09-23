//! illumos host tree.

pub(crate) mod fs {
    pub(crate) use crate::host::common::advise_huge_pages_unsupported as advise_huge_pages;
    pub(crate) use crate::host::common::filesystem_kind_unknown as filesystem_kind;
    pub(crate) use crate::host::common::invalidate_mapped_output_noop as invalidate_mapped_output;
    pub(crate) use crate::host::common::preallocate_unsupported as preallocate;
    pub(crate) use crate::host::unix::fs::InputBytes;
    pub(crate) use crate::host::unix::fs::create_symlink;
    pub(crate) use crate::host::unix::fs::make_executable;
    pub(crate) use crate::host::unix::fs::may_have_multiple_links;
    pub(crate) use crate::host::unix::fs::path_from_bytes;
    pub(crate) use crate::host::unix::fs::read_input;
    pub(crate) use crate::host::unix::fs::release_input_memory;
}

pub(crate) mod os {
    pub(crate) use crate::host::common::kernel_version_unknown as kernel_version;
    #[cfg(test)]
    pub(crate) use crate::host::unix::os::SANDBOXED;

    pub(crate) const IS_MACOS: bool = false;

    /// On Illumos, the Clang driver inserts a meaningless -C flag before calling any non-GNU ld
    /// linker.
    pub(crate) const CLANG_DRIVER_NOOP_SHORT_FLAGS: &[&str] = &["C"];
}

#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin {
    pub(crate) use crate::host::unix::linker_plugin::OffT;
    pub(crate) use crate::host::unix::linker_plugin::PluginLibrary;
    pub(crate) use crate::host::unix::linker_plugin::SUPPORTED;
    pub(crate) use crate::host::unix::linker_plugin::file_descriptor;
    pub(crate) use crate::host::unix::linker_plugin::increase_file_limit;
}

pub(crate) mod perf {
    pub(crate) use crate::host::common::UnsupportedCounterList as CounterList;
}

pub(crate) mod process {
    pub(crate) use crate::host::unix::process::CAN_FORK;
    pub(crate) use crate::host::unix::process::ParentNotifier;
    pub(crate) use crate::host::unix::process::fork_linker;
}
