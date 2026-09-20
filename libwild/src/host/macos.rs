//! macOS host tree.

pub(crate) mod perf {
    pub(crate) use crate::host::common::UnsupportedCounterList as CounterList;
}

pub(crate) mod process {
    pub(crate) use crate::host::unix::process::CAN_FORK;
    pub(crate) use crate::host::unix::process::ParentNotifier;
    pub(crate) use crate::host::unix::process::fork_linker;
}
