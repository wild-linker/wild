//! Facts about the host the linker is running on.

pub(crate) use super::imp::os::CLANG_DRIVER_NOOP_SHORT_FLAGS;
pub(crate) use super::imp::os::IS_MACOS;
/// Whether Rayon can spawn worker threads on the host platform.
pub(crate) use super::imp::os::RAYON_SUPPORTS_MULTI_THREADING;
#[cfg(test)]
pub(crate) use super::imp::os::SANDBOXED;
pub(crate) use super::imp::os::kernel_version;
