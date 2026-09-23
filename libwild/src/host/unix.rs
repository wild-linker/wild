//! Building blocks shared by every unix host tree. Each unix leaf tree re-exports the items it
//! doesn't override.

pub(crate) mod fs {
    use crate::error::Result;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;

    pub(crate) type InputBytes = memmap2::Mmap;

    pub(crate) fn may_have_multiple_links(metadata: &std::fs::Metadata) -> bool {
        use std::os::unix::fs::MetadataExt as _;
        metadata.nlink() > 1
    }

    pub(crate) fn read_input(file: &File, path: &Path, prepopulate: bool) -> Result<InputBytes> {
        crate::host::common::map_input(file, path, prepopulate)
    }

    /// Hints that an input won't be read again.
    pub(crate) fn release_input_memory(bytes: &InputBytes) {
        // Safety: read-only file-backed mapping. Discarded pages can be faulted back in.
        let _ = unsafe { bytes.unchecked_advise(memmap2::UncheckedAdvice::DontNeed) };
    }

    /// Adds execute permission wherever the file currently has read permission.
    pub(crate) fn make_executable(file: &File) -> Result {
        use std::os::unix::prelude::PermissionsExt;
        let mut permissions = file.metadata()?.permissions();
        let mut mode = PermissionsExt::mode(&permissions);
        // Set execute permission wherever we currently have read permission.
        mode = mode | ((mode & 0o444) >> 2);
        PermissionsExt::set_mode(&mut permissions, mode);
        file.set_permissions(permissions)?;
        Ok(())
    }

    pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;
        std::path::Path::new(OsStr::from_bytes(bytes)).to_path_buf()
    }

    pub(crate) fn create_symlink(target: &Path, dest_path: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, dest_path)
    }
}

pub(crate) mod os {
    /// Whether the host is a capability sandbox (no temporary directory, no subprocesses, only
    /// preopened directories).
    #[cfg(test)]
    pub(crate) const SANDBOXED: bool = false;
}

#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin {
    use crate::error::Result;
    use std::ffi::c_int;
    use std::fs::File;

    /// Whether this host can load GCC-compatible LTO linker plugins.
    pub(crate) const SUPPORTED: bool = true;

    /// C `off_t`, as used by the linker plugin API.
    pub(crate) type OffT = libc::off_t;

    /// The raw file descriptor handed to the plugin.
    pub(crate) fn file_descriptor(file: &File) -> c_int {
        std::os::fd::AsRawFd::as_raw_fd(file)
    }

    /// Increase the soft file limit to whatever the hard limit is set to.
    pub(crate) fn increase_file_limit() -> Result {
        use nix::sys::resource::Resource::RLIMIT_NOFILE;

        let (_, hard_limit) = nix::sys::resource::getrlimit(RLIMIT_NOFILE)?;

        nix::sys::resource::setrlimit(RLIMIT_NOFILE, hard_limit, hard_limit)?;

        Ok(())
    }

    /// A loaded linker plugin. Dropping it unloads the plugin.
    pub(crate) struct PluginLibrary(libloading::Library);

    impl PluginLibrary {
        pub(crate) fn open(path: &std::path::Path) -> Result<Self> {
            use crate::error::Context as _;
            // Safety: Truthfully, we don't control the file we're loading. The user gave it to us
            // and there's nothing we can do to guarantee that loading and running it won't trigger
            // UB. The best we can say is that we at least try to conform to the expected plugin
            // API.
            let lib = unsafe { libloading::Library::new(path) }
                .map_err(|e| crate::error!("{}", std::error::Error::source(&e).unwrap_or(&e)))
                .context("Failed to open linker plugin")?;
            Ok(PluginLibrary(lib))
        }

        /// Looks up `name`, which must be of type `T`.
        ///
        /// # Safety
        /// `T` must match the symbol's actual type, and the result must not be used after this
        /// library is dropped.
        pub(crate) unsafe fn symbol<T: Copy>(&self, name: &[u8]) -> Result<T> {
            let symbol = unsafe { self.0.get::<T>(name) }?;
            Ok(*symbol)
        }
    }
}

pub(crate) mod process {
    use crate::bail;
    use crate::error::Context as _;
    use crate::error::Result;
    use crate::host::process::LinkerFork;
    use libc::c_char;
    use libc::pid_t;
    use std::ffi::c_int;
    use std::ffi::c_void;

    pub(crate) const CAN_FORK: bool = true;

    /// Lets the forked linker tell its parent that the link succeeded.
    pub(crate) struct ParentNotifier {
        fds: [c_int; 2],
    }

    impl ParentNotifier {
        /// Inform the parent process that work of linker is done and that it succeeded.
        pub(crate) fn notify_done(&self) {
            let fds = &self.fds;
            unsafe {
                libc::close(fds[0]);
                let stream = libc::fdopen(fds[1], "w".as_ptr().cast::<c_char>());
                let bytes: [u8; 1] = *b"X";
                libc::fwrite(bytes.as_ptr().cast::<c_void>(), 1, 1, stream);
                libc::fclose(stream);
                libc::close(libc::STDOUT_FILENO);
                libc::close(libc::STDERR_FILENO);
            }
        }
    }

    /// Forks. The child links; the parent waits until the child reports that the output has been
    /// written, or exits.
    ///
    /// # Safety
    /// Must not be called once threads have been spawned.
    pub(crate) unsafe fn fork_linker() -> Result<LinkerFork> {
        let mut fds: [c_int; 2] = [0; 2];
        // create the pipe used to communicate between the parent and child processes - exit on
        // failure
        make_pipe(&mut fds).context("make_pipe")?;

        // Safety: our caller guarantees that threads have not yet been started.
        Ok(match unsafe { libc::fork() } {
            0 => LinkerFork::Child(ParentNotifier { fds }),
            -1 => LinkerFork::Failed,
            pid => LinkerFork::Parent(wait_for_child_done(&fds, pid)),
        })
    }

    /// Wait for the child process to signal it is done, by sending a byte on the pipe. In the case
    /// the child crashes, or exits via some path that doesn't send a byte, then the pipe will be
    /// closed and we'll then wait for the subprocess to exit, returning its exit code.
    fn wait_for_child_done(fds: &[c_int], child_pid: pid_t) -> i32 {
        unsafe {
            // close our sending end of the pipe
            libc::close(fds[1]);
            // open the other end of the pipe for reading
            let stream = libc::fdopen(fds[0], "r".as_ptr().cast::<c_char>());

            // Wait for child to send a byte via the pipe or for the pipe to be closed.
            let mut response: [u8; 1] = [0u8; 1];
            if libc::fread(response.as_mut_ptr().cast::<c_void>(), 1, 1, stream) == 1 {
                // Child sent a byte, which indicates that it succeeded and is now shutting down in
                // the background.
                0
            } else {
                // Child closed pipe without sending a byte - get the process exit_status
                let mut status: libc::c_int = -1i32;
                libc::waitpid(child_pid, &raw mut status, 0);
                libc::WEXITSTATUS(status)
            }
        }
    }

    /// Create a pipe for communication between parent and child processes.
    /// If successful it will return Ok and `fds` will have file descriptors for reading and writing
    /// If errors it will return an error message with the errno set, if it can be read or -1 if not
    fn make_pipe(fds: &mut [c_int; 2]) -> Result {
        match unsafe { libc::pipe(fds.as_mut_ptr()) } {
            0 => Ok(()),
            _ => bail!(
                "Error creating pipe. Errno = {:?}",
                std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
            ),
        }
    }
}
