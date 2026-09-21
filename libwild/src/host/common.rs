//! Implementations shared by several host trees, including stubs for services that a host doesn't
//! provide. Each tree re-exports the items it needs, so on any one host some of these are unused.
#![allow(dead_code)]

use crate::args::CounterKind;
use crate::error::Context as _;
use crate::error::Result;
use crate::host::fs::FilesystemKind;
use crate::host::process::LinkerFork;
use std::fs::File;
use std::path::Path;

/// Memory-maps an input file read-only.
pub(crate) fn map_input(file: &File, path: &Path, prepopulate: bool) -> Result<memmap2::Mmap> {
    // Safety: Unfortunately, this is a bit of a compromise. Basically this is only safe if our
    // users manage to avoid editing the input files while we've got them mapped. It'd be great
    // if there were a way to protect against unsoundness when the input files were modified
    // externally, but there isn't - at least on Linux. Not only could the bytes change without
    // notice, but the mapped file could be truncated causing any access to result in a SIGBUS.
    //
    // For our use case, mmap just has too many advantages. There are likely large parts of our
    // input files that we don't need to read, so reading all our input files up front isn't really
    // an option. Reading just the parts we need might be an option, but would add substantial
    // complexity. Also, using mmap means that if the system needs to reclaim memory, it can just
    // release some of our pages.

    let mut mmap_options = memmap2::MmapOptions::new();

    // Prepopulating maps generally slows things down, so is off by default, however it's useful
    // when profiling, since it means that you don't see false positive slowness in the parts of the
    // code that first read a bit of memory.
    if prepopulate {
        mmap_options.populate();
    }

    unsafe { mmap_options.map(file) }
        .with_context(|| format!("Failed to mmap input file `{}`", path.display()))
}

/// `preallocate` for hosts without `fallocate`.
pub(crate) fn preallocate_unsupported(_file: &File, _size: u64) -> Result {
    Err(crate::error!("fallocate is only supported on Linux"))
}

/// `advise_huge_pages` for hosts without `MADV_HUGEPAGE`.
pub(crate) fn advise_huge_pages_unsupported(_mmap: &memmap2::MmapMut) -> Result {
    Err(crate::error!("MADV_HUGEPAGE is only supported on Linux"))
}

/// `invalidate_mapped_output` for hosts with nothing to invalidate.
pub(crate) fn invalidate_mapped_output_noop(_mmap: &mut memmap2::MmapMut, _len: usize) {}

/// `filesystem_kind` for hosts where detection wouldn't change the output defaults.
pub(crate) fn filesystem_kind_unknown(_file: &File) -> Option<FilesystemKind> {
    None
}

/// `kernel_version` for hosts where the output defaults don't depend on it.
pub(crate) fn kernel_version_unknown() -> Option<(u64, u64)> {
    None
}

/// `PluginLibrary` for hosts or builds that can't load linker plugins. Nothing tries to open one,
/// since `SUPPORTED` is false on those hosts and the `plugins` feature is off in those builds.
pub(crate) struct UnsupportedPluginLibrary;

impl UnsupportedPluginLibrary {
    pub(crate) fn open(_path: &Path) -> Result<Self> {
        Err(crate::error!(
            "Linker plugins are not supported on this host"
        ))
    }

    /// # Safety
    /// See `PluginLibrary::symbol` on hosts that support plugins.
    #[allow(clippy::unused_self)]
    pub(crate) unsafe fn symbol<T: Copy>(&self, _name: &[u8]) -> Result<T> {
        Err(crate::error!(
            "Linker plugins are not supported on this host"
        ))
    }
}

/// `CounterList` for hosts without performance counters. It never reports any counters.
pub(crate) struct UnsupportedCounterList {}

impl UnsupportedCounterList {
    pub(crate) fn from_kinds(_opts: &[CounterKind]) -> Self {
        UnsupportedCounterList {}
    }

    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) fn read(&mut self) -> Vec<Option<crate::timing::CounterSnapshot>> {
        Vec::new()
    }
}

/// Process services for hosts that can't fork.
pub(crate) mod no_fork {
    use super::LinkerFork;
    use crate::error::Result;

    pub(crate) const CAN_FORK: bool = false;

    pub(crate) struct ParentNotifier;

    impl ParentNotifier {
        #[allow(clippy::unused_self)]
        pub(crate) fn notify_done(&self) {}
    }

    /// # Safety
    /// Always safe; the signature matches the hosts that can fork.
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) unsafe fn fork_linker() -> Result<LinkerFork> {
        Ok(LinkerFork::Failed)
    }
}
