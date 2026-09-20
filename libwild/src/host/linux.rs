//! Linux and Android host tree.

pub(crate) mod process {
    pub(crate) use crate::host::unix::process::CAN_FORK;
    pub(crate) use crate::host::unix::process::ParentNotifier;
    pub(crate) use crate::host::unix::process::fork_linker;
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
