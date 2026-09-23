//! Linux and Android host tree.

pub(crate) mod fs {
    pub(crate) use crate::host::common::invalidate_mapped_output_noop as invalidate_mapped_output;
    use crate::host::fs::FilesystemKind;
    pub(crate) use crate::host::unix::fs::InputBytes;
    pub(crate) use crate::host::unix::fs::create_symlink;
    pub(crate) use crate::host::unix::fs::make_executable;
    pub(crate) use crate::host::unix::fs::may_have_multiple_links;
    pub(crate) use crate::host::unix::fs::path_from_bytes;
    pub(crate) use crate::host::unix::fs::read_input;
    pub(crate) use crate::host::unix::fs::release_input_memory;
    use std::fs::File;

    cfg_select! {
        target_os = "linux" => {
            pub(crate) fn preallocate(file: &File, size: u64) -> crate::error::Result {
                use crate::error::Context as _;

                if size > 0 {
                    nix::fcntl::fallocate(
                        file,
                        nix::fcntl::FallocateFlags::empty(),
                        0,
                        i64::try_from(size).context("Output file is too large for fallocate")?,
                    )?;
                }

                Ok(())
            }

            pub(crate) fn advise_huge_pages(mmap: &memmap2::MmapMut) -> crate::error::Result {
                mmap.advise(memmap2::Advice::HugePage)?;
                Ok(())
            }
        }
        _ => {
            // Android: nix and memmap2 don't expose fallocate or MADV_HUGEPAGE there.
            pub(crate) use crate::host::common::advise_huge_pages_unsupported as advise_huge_pages;
            pub(crate) use crate::host::common::preallocate_unsupported as preallocate;
        }
    }

    pub(crate) fn filesystem_kind(file: &File) -> Option<FilesystemKind> {
        use nix::sys::statfs;

        let fs_type = statfs::fstatfs(file).ok()?.filesystem_type();

        Some(match fs_type {
            statfs::BTRFS_SUPER_MAGIC => FilesystemKind::Btrfs,
            statfs::MSDOS_SUPER_MAGIC => FilesystemKind::Vfat,
            // Note, despite the constant name, this actually applies to ext3 and ext2 as well as
            // ext4.
            statfs::EXT4_SUPER_MAGIC => FilesystemKind::Ext4,
            // For some reason statfs doesn't define the XFS constant when target is musl.
            #[cfg(not(target_env = "musl"))]
            statfs::XFS_SUPER_MAGIC => FilesystemKind::Xfs,
            _ => FilesystemKind::Other,
        })
    }
}

pub(crate) mod os {
    #[cfg(test)]
    pub(crate) use crate::host::unix::os::SANDBOXED;

    pub(crate) const IS_MACOS: bool = false;

    pub(crate) const CLANG_DRIVER_NOOP_SHORT_FLAGS: &[&str] = &[];

    pub(crate) fn kernel_version() -> Option<(u64, u64)> {
        let uname = nix::sys::utsname::uname().ok()?;
        let release = uname.release().to_string_lossy();
        let mut parts = release.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;

        Some((major, minor))
    }
}

#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin {
    pub(crate) use crate::host::unix::linker_plugin::OffT;
    pub(crate) use crate::host::unix::linker_plugin::PluginLibrary;
    pub(crate) use crate::host::unix::linker_plugin::SUPPORTED;
    pub(crate) use crate::host::unix::linker_plugin::file_descriptor;
    pub(crate) use crate::host::unix::linker_plugin::increase_file_limit;
}

cfg_select! {
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ) => {
        #[path = "linux/perf.rs"]
        pub(crate) mod perf;
    }
    _ => {
        pub(crate) mod perf {
            pub(crate) use crate::host::common::UnsupportedCounterList as CounterList;
        }
    }
}

pub(crate) mod process {
    pub(crate) use crate::host::unix::process::CAN_FORK;
    pub(crate) use crate::host::unix::process::ParentNotifier;
    pub(crate) use crate::host::unix::process::fork_linker;
}
