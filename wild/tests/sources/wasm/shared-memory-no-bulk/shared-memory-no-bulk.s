//#LinkArgs: --shared-memory --max-memory=131072 --no-entry
//#ExpectError: 'bulk-memory' feature must be used in order to use shared memory

//#Config:no-check
//#LinkArgs: --shared-memory --max-memory=131072 --no-entry --no-check-features
//#RunEnabled:false
//#ExpectSharedMemory:true

//#Config:features
//#LinkArgs: --shared-memory --max-memory=131072 --no-entry --features=atomics,bulk-memory
//#RunEnabled:false
//#ExpectSharedMemory:true
//#ExpectSection:target_features
//#Contains:atomics
//#Contains:bulk-memory

//#Config:plus
//#LinkArgs: --shared-memory --max-memory=131072 --no-entry --features=+atomics,+bulk-memory
//#ExpectError: 'atomics' feature must be used in order to use shared memory

//#Config:literal-minus
//#LinkArgs: --shared-memory --max-memory=131072 --no-entry --no-check-features --features=-shared-mem
//#RunEnabled:false
//#ExpectSharedMemory:true
//#Contains:-shared-mem

  .globl _start
_start:
  .functype _start () -> ()
  end_function

  .section .custom_section.target_features,"",@
  .int8 1 # Number of target features
  .int8 43 # '+'
  .int8 7 # Length of target features string
  .ascii "atomics"
