(module
  (memory 1)
  (data "\02\00\00\00")
  (func $check_other (export "check_other")
    i32.const 0
    i32.const 0
    i32.const 4
    memory.init 0
    i32.const 0
    i32.load
    i32.const 2
    i32.ne
    if
      unreachable
    end
  )
)
