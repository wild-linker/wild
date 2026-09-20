//! Building blocks shared by every unix host tree. Each unix leaf tree re-exports the items it
//! doesn't override.

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
