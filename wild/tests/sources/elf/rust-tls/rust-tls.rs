//#AbstractConfig:default
// LLD 23 removes the high-part TLSLE addition when the offset fits in 12 bits.
// TODO: Implement this relaxation in Wild.
//#DiffIgnore:rel.missing-opt.R_AARCH64_TLSLE_ADD_TPREL_HI12.ReplaceWithNop.*
//#RequiresNightlyRustc:true
// TODO: Investigate the following differences, which show up on risc-v
//#DiffIgnore:dynsym.*
//#DiffIgnore:.dynamic.DT_FLAGS.TEXTREL
//#DiffIgnore:.dynamic.DT_TEXTREL

//#Config:global-dynamic:default
//#CompArgs:-Ztls-model=global-dynamic

//#Config:local-dynamic:default
//#CompArgs:-Ztls-model=local-dynamic

//#Config:initial-exec:default
//#CompArgs:-Ztls-model=initial-exec

//#Config:local-exec:default
//#CompArgs:-Ztls-model=local-exec

use std::cell::Cell;
use std::thread;

thread_local!(static FOO: Cell<u32> = Cell::new(1));

fn main() {
    assert_eq!(FOO.get(), 1);
    FOO.set(2);

    // each thread starts out with the initial value of 1
    let t = thread::spawn(move || {
        assert_eq!(FOO.get(), 1);
        FOO.set(3);
    });

    // wait for the thread to complete and bail out on panic
    t.join().unwrap();

    // we retain our original value of 2 despite the child thread
    assert_eq!(FOO.get(), 2);

    std::process::exit(42);
}
