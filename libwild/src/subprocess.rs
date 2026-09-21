use crate::Args;
use crate::error::Result;
use crate::host::process::LinkerFork;

/// Runs the linker, in a subprocess if possible, prints any errors, then exits.
///
/// This is done by forking a sub-process which runs the linker and waits for communication back
/// from the sub-process (via a pipe) when the main link task is done (the output file has been
/// written, but some shutdown tasks remain.
///
/// Don't call `setup_tracing` or `setup_thread_pool` if using this function, these will be called
/// for you in the subprocess.
///
/// # Safety
/// Must not be called once threads have been spawned. Calling this function from main is generally
/// the best way to ensure this.
pub unsafe fn run_in_subprocess(args: Args) -> ! {
    if !cfg!(feature = "fork") || !crate::host::process::CAN_FORK {
        let exit_code = match crate::run(args) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("{}", error.to_string());
                -1
            }
        };
        std::process::exit(exit_code);
    }

    let exit_code = match subprocess_result(args) {
        Ok(code) => code,
        Err(error) => crate::error::report_error_and_exit(&error),
    };
    std::process::exit(exit_code);
}

fn subprocess_result(mut args: Args) -> Result<i32> {
    // Safety: The function we're in is private to this module and is only called from
    // run_in_subprocess, which imposed the requirement that threads have not yet been started on
    // its caller.
    match unsafe { crate::host::process::fork_linker() }? {
        LinkerFork::Child(parent) => {
            // Fork success in child - Run linker in this process.

            crate::setup_tracing(&args)?;
            let thread_pool = args.common_mut().build_thread_pool()?;
            thread_pool.pool.install(|| -> Result {
                let linker = crate::Linker::new();
                let _outputs = linker.run(&args)?;
                crate::timing::finalise_perfetto_trace()?;
                parent.notify_done();
                Ok(())
            })?;
            Ok(0)
        }
        LinkerFork::Failed => {
            // Fork failure in the parent - Fallback to running linker in this process

            crate::run(args)?;
            Ok(0)
        }
        LinkerFork::Parent(exit_status) => {
            // Fork success in the parent - the child has signalled us that it's done
            Ok(exit_status)
        }
    }
}
