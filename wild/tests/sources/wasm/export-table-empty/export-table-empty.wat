;;#Config:default
;;#NoSection:Table
;;#NoSection:Element
;;#NoSym:__indirect_function_table
;;#ExpectSym:memory

;;#Config:export-table
;;#LinkArgs:--export-table
;;#ExpectSection:Table
;;#NoSection:Element
;;#ExpectSym:__indirect_function_table
;;#ExpectSym:memory

(module
  (func (export "_start"))
)
