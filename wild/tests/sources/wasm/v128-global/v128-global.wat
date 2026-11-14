;; wasm-ld rejects v128.const in global initializers.
;;#ReferenceLinkers:

(module
  (global $value v128 (v128.const i32x4 42 0 0 0))
  (func $_start (export "_start")
    global.get $value
    i32x4.extract_lane 0
    i32.const 42
    i32.ne
    if unreachable end
  )
)
