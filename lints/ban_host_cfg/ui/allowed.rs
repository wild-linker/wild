// Neutral code: features, tests, and selecting what we're linking for are all fine, as is
// reaching host services through the facades.
mod host {
    pub mod fs {
        pub fn path_from_bytes(_bytes: &[u8]) {}
    }
}

#[cfg(feature = "plugins")]
fn with_plugins() {}

#[cfg(all(test, debug_assertions))]
fn in_tests() {}

#[cfg(target_arch = "aarch64")]
fn for_aarch64_target() {}

fn uses_the_facade() {
    crate::host::fs::path_from_bytes(b"/tmp");
}

fn main() {
    uses_the_facade();
}
