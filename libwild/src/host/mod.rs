//! Services that depend on the host the linker runs on.
//!
//! This module is the only place in libwild that selects code based on the *host* operating system
//! (the machine the linker runs on). The host is unrelated to the linker *target*: wild can link
//! for a target other than the one it's running on, so output-format and architecture selection
//! live elsewhere and never use host `cfg`. It's also unrelated to `crate::platform`, which is the
//! linker-format (ELF, Mach-O, …) trait.
//!
//! The `cfg_select!` below picks exactly one host tree and aliases it as `imp`. It deliberately
//! has no `_` fallback arm. Like a C/C++ `#if … #elif … #else #error` chain, building for a host
//! that isn't listed is a compile error that points here, rather than silently inheriting whatever
//! the scattered `cfg(not(...))` branches happened to do.
//!
//! The facade modules (`fs`, `os`, `linker_plugin`, `perf`, `process`) define host-independent
//! types and re-export the selected tree's items. Every tree must provide every item, so adding a
//! host service is a compile error in each tree until it's implemented or explicitly stubbed, and
//! the rest of the crate never names a host.

mod common;
pub(crate) mod fs;
#[cfg(feature = "plugins")]
pub(crate) mod linker_plugin;
pub(crate) mod os;
pub(crate) mod perf;
pub(crate) mod process;

cfg_select! {
    windows => {
        mod windows;
        use windows as imp;
    }
    target_os = "wasi" => {
        mod wasi;
        use wasi as imp;
    }
    any(target_os = "linux", target_os = "android") => {
        mod unix;
        mod linux;
        use linux as imp;
    }
    target_os = "macos" => {
        mod unix;
        mod macos;
        use macos as imp;
    }
    target_os = "illumos" => {
        mod unix;
        mod illumos;
        use illumos as imp;
    }
    unix => {
        mod unix;
        mod other_unix;
        use other_unix as imp;
    }
}
