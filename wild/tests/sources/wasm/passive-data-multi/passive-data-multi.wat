;;#Object:passive-data-multi2.wat
;;#ReferenceLinkers:
;;#ExpectSection:DataCount

(module
  (import "env" "check_other" (func $check_other))
  (memory (export "memory") 1)
  (data "\01\00\00\00")
  (func $_start (export "_start")
    call $check_other
    i32.const 8
    i32.const 0
    i32.const 4
    memory.init 0
    i32.const 8
    i32.load
    i32.const 1
    i32.ne
    if
      unreachable
    end
  )
)
