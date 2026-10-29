//#AbstractConfig:base
//#Arch:x86_64
//#Mode:static
//#RunEnabled:false
//#ReferenceLinkers:bfd
//#LinkArgs:-e _start

//#AbstractConfig:rwx:base
//#ExpectProgramHeader:LOAD flags=RWX,sections=[.wtext]

//#Config:default:rwx
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:warn:rwx
//#LinkArgs:--no-warn-rwx-segments --warn-rwx-segments
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:no-warn:rwx
//#LinkArgs:--warn-rwx-segments --no-warn-rwx-segments
//#ExpectWarning:^$

//#Config:error:rwx
//#LinkArgs:--error-rwx-segments
//#ExpectError:has a LOAD segment with RWX permissions

//#Config:no-error:rwx
//#LinkArgs:--error-rwx-segments --no-error-rwx-segments
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:no-error-default:rwx
//#LinkArgs:--no-error-rwx-segments
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:no-error-disabled:rwx
//#LinkArgs:--no-warn-rwx-segments --no-error-rwx-segments
//#ExpectWarning:^$

//#Config:error-last:rwx
//#LinkArgs:--no-error-rwx-segments --error-rwx-segments
//#ExpectError:has a LOAD segment with RWX permissions

//#Config:error-disabled:rwx
//#LinkArgs:--error-rwx-segments --no-warn-rwx-segments
//#ExpectWarning:^$

//#Config:error-enables:rwx
// Wild, unlike BFD, treats warn and error as a single setting with last-flag wins.
//#ReferenceLinkers:
//#LinkArgs:--no-warn-rwx-segments --error-rwx-segments
//#ExpectError:has a LOAD segment with RWX permissions

//#Config:explicit-rx:base
//#LinkArgs:-T ./rx.ld --error-rwx-segments
//#ExpectProgramHeader:LOAD flags=RX,sections=[.wtext,*]
//#ExpectWarning:^$

//#Config:relocatable:base
//#LinkArgs:-r --error-rwx-segments
//#DiffEnabled:false
//#NoProgramHeader:LOAD
//#ExpectWarning:^$

.section .wtext,"awx"
.globl _start
_start:
    ret

.section .note.GNU-stack,"",@progbits
