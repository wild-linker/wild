//! Host process services: forking the linker so that the parent can exit while the child finishes
//! shutting down.

pub(crate) use super::platform_imp::process::CAN_FORK;
pub(crate) use super::platform_imp::process::ParentNotifier;
pub(crate) use super::platform_imp::process::fork_linker;

/// The outcome of [`fork_linker`].
// `Child` and `Parent` are only constructed on hosts that can fork.
#[allow(dead_code)]
pub(crate) enum LinkerFork {
    /// Running in the forked child. Call `ParentNotifier::notify_done` once outputs are written.
    Child(ParentNotifier),
    /// Running in the parent after the child reported back. The value is the exit code to use.
    Parent(i32),
    /// `fork` failed. We're still the only process, so link in this one.
    Failed,
}
