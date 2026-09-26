// TODO: For now, LTO is required for this test since we have a single page limit.
// We also require `-link-dead-code=y` and the `__eh_frame` DiffIgnore assertion since we don't have
// `-dead_strip` support yet.
//#CompArgs:-C link-dead-code=y -C lto=y -C opt-level=2
//#DiffIgnore:section.__eh_frame.attributes
// lld and ld both use lazy binding, but we use chained fixups here.
//#DiffIgnore:section.__stub_helper
//#DiffIgnore:section.__la_symbol_ptr
//#ExpectSym:_rust_eh_personality section="__text"

fn main() {
    assert!(std::panic::catch_unwind(|| panic!("Local personality")).is_err());
    std::process::exit(42);
}
