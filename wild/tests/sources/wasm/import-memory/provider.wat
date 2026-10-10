(module
  (memory (export "memory") 2 3 shared)
  (func (export "read_at") (param $addr i32) (result i32)
    local.get $addr
    i32.load)
  (func (export "write_at") (param $addr i32) (param $value i32)
    local.get $addr
    local.get $value
    i32.store))
