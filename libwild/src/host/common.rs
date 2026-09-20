//! Implementations shared by several host trees, including stubs for services that a host doesn't
//! provide. Each tree re-exports the items it needs, so on any one host some of these are unused.
#![allow(dead_code)]

use crate::args::CounterKind;
use crate::host::process::LinkerFork;

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
