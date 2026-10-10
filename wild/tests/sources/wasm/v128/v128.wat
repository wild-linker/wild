(module
  (func $add_v128 (export "add_v128") (param v128 v128) (result v128)
    local.get 0
    local.get 1
    i32x4.add
  )
  (func $start (export "_start")
    (local $sum v128)
    v128.const i32x4 1 1 1 1
    v128.const i32x4 1 2 3 4
    call $add_v128
    local.set $sum

    local.get $sum
    i32x4.extract_lane 0
    i32.const 2
    i32.ne
    if unreachable end

    local.get $sum
    i32x4.extract_lane 1
    i32.const 3
    i32.ne
    if unreachable end

    local.get $sum
    i32x4.extract_lane 2
    i32.const 4
    i32.ne
    if unreachable end

    local.get $sum
    i32x4.extract_lane 3
    i32.const 5
    i32.ne
    if unreachable end
  )
)
