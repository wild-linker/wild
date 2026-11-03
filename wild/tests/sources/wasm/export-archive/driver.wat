(module
  (import "test" "exported_from_archive" (func $exported_from_archive (result i32)))
  (func (export "_start")
    call $exported_from_archive
    i32.const 42
    i32.ne
    if
      unreachable
    end))
