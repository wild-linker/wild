// Each finding is reported against the file's first item, so that item is kept small here.
fn main() {
    let _ = runtime_check();
    names_a_tree();
}

mod host {
    pub mod linux {
        pub mod fs {
            pub fn preallocate() {}
        }
    }
}

#[cfg(unix)]
fn only_on_unix() {}

#[cfg_attr(target_os = "macos", allow(dead_code))]
fn macos_attr() {}

fn runtime_check() -> bool {
    cfg!(windows)
}

fn native_api(file: &std::fs::File) -> i32 {
    std::os::fd::AsRawFd::as_raw_fd(file)
}

fn names_a_tree() {
    crate::host::linux::fs::preallocate();
}
