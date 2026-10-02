;;#ReferenceLinkers:
;;#RunEnabled:false
;;#NoSection:Data
;;#NoSection:DataCount
;;#DoesNotContain:UNREFERENCED_PASSIVE_SEGMENT

(module
  (memory (export "memory") 1)
  (data "UNREFERENCED_PASSIVE_SEGMENT")
  (func $dead
    i32.const 0
    i32.const 0
    i32.const 4
    memory.init 0
  )
  (func $_start (export "_start")
    nop
  )
)
