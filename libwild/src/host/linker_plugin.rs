//! Host support for GCC-compatible LTO linker plugins.

pub(crate) use super::imp::linker_plugin::OffT;
pub(crate) use super::imp::linker_plugin::PluginLibrary;
pub(crate) use super::imp::linker_plugin::SUPPORTED;
pub(crate) use super::imp::linker_plugin::file_descriptor;
pub(crate) use super::imp::linker_plugin::increase_file_limit;
