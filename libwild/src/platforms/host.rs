//! Facts about the host the linker is running on.

pub(crate) use super::platform_imp::host::CLANG_DRIVER_NOOP_SHORT_FLAGS;
pub(crate) use super::platform_imp::host::IS_MACOS;
#[cfg(test)]
pub(crate) use super::platform_imp::host::SANDBOXED;
pub(crate) use super::platform_imp::host::kernel_version;
