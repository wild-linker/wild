//#AbstractConfig:base
//#Arch:x86_64
//#Mode:static
//#RunEnabled:false
//#ReferenceLinkers:bfd
//#LinkArgs:--no-gc-sections -e _start

//#Config:combined-load:base
//#LinkArgs:-T ./combined.ld
//#ExpectProgramHeader:LOAD flags=RWX,sections=[.text,.data,.bss,*]
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:combined-error:base
//#LinkArgs:-T ./combined.ld --error-rwx-segments
//#ExpectError:has a LOAD segment with RWX permissions

//#Config:multiple-errors:base
//#ReferenceLinkers:
//#Object:tls.s
//#LinkArgs:-T ./multiple-errors.ld --error-rwx-segments
//#ExpectError:(?m)^(wild: error: .+ has a LOAD segment with RWX permissions\n){2}
//#ExpectError:(?m)^wild: error: .+ has a TLS segment with execute permission$

//#Config:merged-section:base
//#LinkArgs:-T ./merged.ld
//#ExpectProgramHeader:LOAD flags=RWX,sections=[.merged,*]
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:bss-only:base
//#LinkArgs:-T ./bss.ld
//#ExpectProgramHeader:LOAD flags=RWX,sections=[.bss,*],file-size=0,mem-size=8
//#ExpectWarning:has a LOAD segment with RWX permissions

//#Config:empty:base
//#LinkArgs:-T ./empty.ld --error-rwx-segments
//#ExpectProgramHeader:LOAD flags=RWX,mem-size=0
//#ExpectProgramHeader:TLS flags=X,mem-size=0
//#DiffIgnore:segment.TLS.alignment

//#Config:tls:base
//#Object:tls.s
//#LinkArgs:-T ./tls.ld
//#ExpectProgramHeader:TLS flags=X,sections=[.tdata]
//#ExpectWarning:has a TLS segment with execute permission

//#Config:tls-error:base
//#Object:tls.s
//#LinkArgs:-T ./tls.ld --error-rwx-segments
//#ExpectError:has a TLS segment with execute permission

//#Config:tls-disabled:base
//#Object:tls.s
//#LinkArgs:-T ./tls.ld --error-rwx-segments --no-warn-rwx-segments
//#ExpectProgramHeader:TLS flags=X,sections=[.tdata]

//#Config:execstack:base
//#LinkArgs:-z execstack --error-rwx-segments
//#ExpectProgramHeader:GNU_STACK flags=RWX

.text
.globl _start
_start:
    ret

.data
    .quad 1

.bss
    .skip 8

.section .empty,"a",@nobits

.section .note.GNU-stack,"",@progbits
