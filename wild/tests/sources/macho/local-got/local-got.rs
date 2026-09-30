// We require the `__eh_frame` DiffIgnore assertion since we don't have `-dead_strip` support yet.
//#DiffIgnore:section.__eh_frame.attributes
// lld and ld both use lazy binding, but we use chained fixups here.
//#DiffIgnore:section.__stub_helper
//#DiffIgnore:section.__la_symbol_ptr
//#ExpectSym:_rust_eh_personality section="__text"

fn main() {
    assert!(std::panic::catch_unwind(|| panic!("Local personality")).is_err());
    std::process::exit(42);
}
