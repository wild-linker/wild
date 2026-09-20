//#AbstractConfig:default
//#Object:runtime.c
//#Arch: x86_64

//#Config:error:default
//#ReferenceLinkers:
//#ExpectError:requires executable stack, but -z execstack is not specified

//#Config:allowed:default
//#LinkArgs:-z execstack

//#Config:no-error:default
//#ReferenceLinkers:
//#LinkArgs:--no-error-execstack
//#ExpectWarning:requires executable stack, but -z execstack is not specified

//#Config:no-warn:default
//#ReferenceLinkers:bfd
//#LinkArgs:--no-warn-execstack

//#Config:warn-always:default
//#ReferenceLinkers:
//#LinkArgs:-z execstack --warn-execstack --no-error-execstack
//#ExpectWarning:enabling an executable stack because of -z execstack

//#Config:warn-always-error:default
//#ReferenceLinkers:
//#LinkArgs:-z execstack --warn-execstack
//#ExpectError:creating an executable stack because of -z execstack

//#Config:warn-always-error-explicit:default
//#ReferenceLinkers:
//#LinkArgs:-z execstack --warn-execstack --error-execstack
//#ExpectError:creating an executable stack because of -z execstack

//#Config:warn-objects-with-z:default
//#ReferenceLinkers:bfd
//#LinkArgs:-z execstack --warn-execstack-objects --error-execstack

//#Config:error-explicit:default
//#ReferenceLinkers:
//#LinkArgs:--error-execstack --warn-execstack-objects
//#ExpectError:requires executable stack, but -z execstack is not specified

.globl _start
_start:
    mov     $42, %rdi
    call    exit_syscall

.section .note.GNU-stack,"x",@progbits
