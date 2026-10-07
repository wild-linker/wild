(module
  (import "test" "__wasm_call_ctors" (func $call_ctors))
  (import "test" "foo" (func $foo))
  (import "test" "add" (func $add (param i32 i32) (result i32)))
  (func (export "_start")
    call $call_ctors
    call $foo
    i32.const 19
    i32.const 23
    call $add
    i32.const 42
    i32.ne
    if
      unreachable
    end))
