//! Facts about the host the linker is running on.

pub(crate) use super::imp::os::CLANG_DRIVER_NOOP_SHORT_FLAGS;
pub(crate) use super::imp::os::IS_MACOS;
#[cfg(test)]
pub(crate) use super::imp::os::SANDBOXED;
pub(crate) use super::imp::os::kernel_version;
