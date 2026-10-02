;; Input passive segments stay passive. DataCount is required for `memory.init`.
;; wasm-ld currently converts the segment to active and omits DataCount, so this
;; is Wild-only.
;;#ReferenceLinkers:
;;#ExpectSection: Data
;;#ExpectSection: DataCount

(module
  (memory (export "memory") 1)
  (data "\2A\00\00\00")
  (func $_start (export "_start")
    i32.const 0
    i32.const 0
    i32.const 4
    memory.init 0
    i32.const 0
    i32.load
    i32.const 42
    i32.ne
    if
      unreachable
    end
  )
)
