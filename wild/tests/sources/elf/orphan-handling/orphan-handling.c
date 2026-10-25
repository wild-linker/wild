//#AbstractConfig:common
//#Object:runtime.c
// RISC-V: BFD complains about missing __global_pointer$ (defined in the default linker script)
//#SkipArch:riscv64
//#ReferenceLinkers:bfd
//#DiffIgnore:segment.LOAD.RX.alignment
//#DiffIgnore:segment.LOAD.RW.alignment
//#DiffIgnore:segment.LOAD.RWX.alignment

//#AbstractConfig:default:common
//#LinkerScript:orphan-handling.ld

//#Config:orphan-place:default
//#LinkArgs:--orphan-handling=place
//#Contains:.orphan_section
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-warn:default
//#LinkArgs:--orphan-handling=warn
//#ExpectWarning:orphan section .\.orphan_section' from .*orphan-handling\.c\.o' being placed in section .\.orphan_section.
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-error:default
//#CompArgs:-fno-asynchronous-unwind-tables -fno-unwind-tables
//#LinkArgs:--orphan-handling=error
//#ExpectError:unplaced orphan section .\.orphan_section' from .*orphan-handling\.c\.o'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-discard:default
//#LinkArgs:--orphan-handling=discard
//#DoesNotContain:.orphan_section
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-debug-place:default
//#CompArgs:-g
//#LinkArgs:--orphan-handling=place
//#ExpectSection:.debug_info
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-debug-discard:default
//#CompArgs:-g
//#LinkArgs:--orphan-handling=discard
//#NoSection:.debug_info
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:eh-frame:default
//#CompArgs:-fasynchronous-unwind-tables -DNO_ORPHAN_VAR

//#Config:orphan-eh-frame-place:eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=place
//#ExpectSection:.eh_frame
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-eh-frame-warn:eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=warn
//#ExpectWarning:orphan section .\.eh_frame'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-eh-frame-error:eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=error
//#ExpectError:unplaced orphan section .\.eh_frame'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:orphan-eh-frame-discard:eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=discard
//#NoSection:.eh_frame
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-orphan-eh-frame-discard:common
//#LinkerScript:orphan-handling-with-ehframe.ld
//#LinkArgs:--no-gc-sections --orphan-handling=discard
//#ExpectSection:.eh_frame
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:filtered-eh-frame:common
//#LinkerScript:orphan-handling-filtered-ehframe.ld
//#CompArgs:-fasynchronous-unwind-tables -DNO_ORPHAN_VAR

//#Config:filtered-eh-frame-place:filtered-eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=place
//#ExpectSection:.eh_frame
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:filtered-eh-frame-warn:filtered-eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=warn
//#ExpectWarning:orphan section .\.eh_frame'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:filtered-eh-frame-error:filtered-eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=error
//#ExpectError:unplaced orphan section .\.eh_frame'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:filtered-eh-frame-discard:filtered-eh-frame
//#LinkArgs:--no-gc-sections --orphan-handling=discard
//#NoSection:.eh_frame
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:partial:common
//#RunEnabled:false
//#DiffEnabled:false

//#Config:orphan-partial-place:partial
//#LinkArgs:-r --orphan-handling=place
//#ExpectSection:.orphan_section
//#CompArgs:-fPIE

//#Config:orphan-partial-warn:partial
//#LinkArgs:-r --orphan-handling=warn
//#ExpectWarning:orphan section .\.orphan_section'
//#CompArgs:-fPIE

//#Config:orphan-partial-error:partial
//#LinkArgs:-r --orphan-handling=error
//#ExpectError:unplaced orphan section .\.orphan_section'
//#CompArgs:-fPIE

//#Config:orphan-partial-discard:partial
//#LinkArgs:-r --orphan-handling=discard
//#NoSection:.orphan_section
//#CompArgs:-fPIE

//#Config:no-script-place:common
//#LinkArgs:--orphan-handling=place
//#ExpectSection:.orphan_section
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-script-warn:common
//#LinkArgs:--orphan-handling=warn
//#ExpectWarning:orphan section .\.orphan_section'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-script-error:common
//#LinkArgs:--orphan-handling=error
//#ExpectError:unplaced orphan section .\.orphan_section'
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-script-discard:common
//#LinkArgs:--orphan-handling=discard
//#NoSection:.orphan_section
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#AbstractConfig:no-orphans:common
//#CompArgs:-Wa,-mx86-used-note=no
// Some other architectures emit platform specific sections which might not be handled by wild, such
// as .ARM.attributes on AArch64.
//#Arch:x86_64

//#Config:no-orphans-error:no-orphans
//#CompArgs:-DNO_ORPHAN_VAR
//#LinkArgs:--orphan-handling=error
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-orphans-warn:no-orphans
//#CompArgs:-DNO_ORPHAN_VAR
//#LinkArgs:--orphan-handling=warn
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:no-orphans-discard:no-orphans
//#CompArgs:-DNO_ORPHAN_VAR
//#LinkArgs:--orphan-handling=discard
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

#include "../common/runtime.h"

#ifndef NO_ORPHAN_VAR
__attribute__((section(".orphan_section"), retain, used)) int orphan_var = 42;
#endif

void _start(void) { exit_syscall(42); }
