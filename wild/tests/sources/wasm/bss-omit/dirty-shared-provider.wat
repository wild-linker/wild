(module
  (memory (export "memory") 2 3 shared)
  (func $dirty
    i32.const 65536
    i32.const 0xaa
    i32.const 4112
    memory.fill)
  (start $dirty))
