;;#Object:merge-data-passive2.s
;;#ReferenceLinkers:
;;#ExpectSection: Data
;;#ExpectSection: DataCount
;;#ExpectDataSegmentCount: 3
;;#ExpectDataSegmentName: 0=.rodata
;;#ExpectDataSegmentName: 1=.data

(module
  (import "env" "check_merged" (func $check_merged))
  (memory (export "memory") 1)
  (data "\2A\00\00\00")
  (func $_start (export "_start")
    call $check_merged
    i32.const 0
    i32.const 0
    i32.const 4
    memory.init 0
    data.drop 0
    i32.const 0
    i32.load
    i32.const 42
    i32.ne
    if
      unreachable
    end
  )
)
