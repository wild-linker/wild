(module
  (import "test" "__wasm_init_tls" (func $init_tls (param i32)))
  (import "test" "__heap_base" (global $heap_base i32))
  (import "test" "check" (func $check))
  (func (export "_start")
    global.get $heap_base
    call $init_tls
    call $check))
