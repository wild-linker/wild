//#AbstractConfig:default
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false
//#CompArgs:-fPIE -fno-asynchronous-unwind-tables
//#LinkArgs:-no-pie --no-gc-sections

//#Config:address_text:default
//#LinkerScript:address-data.ld
//#CompArgs:-DTEST_ADDRESS
//#LinkArgs:-Ttext=0x600000
//#ExpectSym:_start address=0x600000

//#Config:address_data:default
//#LinkerScript:address-text.ld
//#CompArgs:-DTEST_ADDRESS
//#LinkArgs:-Tdata=0x600000
//#ExpectSym:data address=0x600000

//#Config:address_bss:default
//#LinkerScript:address-text.ld
//#CompArgs:-DTEST_ADDRESS
//#LinkArgs:-Tbss=0x600000
//#ExpectSym:bss address=0x600000

//#Config:orphan_data:default
//#LinkerScript:orphan-data.ld
//#ReferenceLinkers:bfd
//#SkipArch:riscv64
//#Object:runtime.c
//#RunEnabled:true
//#CompArgs:-DTEST_ORPHAN_DATA
//#ExpectProgramHeader:LOAD sections=[*,.data,.orphan_data,.bss]
//#ExpectSym:data_var section=.data
//#ExpectSym:orphan_var section=.orphan_data
//#ExpectSym:bss_var section=.bss

//#Config:init_array:default
//#LinkerScript:orphan-init-array.ld
//#CompArgs:-DTEST_INIT_ARRAY
//#ExpectSym:foo section=.text
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

//#Config:loc_counter_tls:default
//#CompArgs:-DTEST_LOC_COUNTER_TLS
//#LinkerScript:loc-counter-tls.ld
//#ExpectSym:initialized section=.tdata
//#ExpectSym:uninitialized section=.tbss

//#Config:loc_counter_data:default
//#CompArgs:-DTEST_LOC_COUNTER_DATA
//#LinkerScript:loc-counter-data.ld
//#ExpectSym:initialized section=.orphan
//#ExpectSym:uninitialized section=.bss

//#Config:merge_address:default
//#CompArgs:-DTEST_MERGE_ADDRESS
//#LinkerScript:merge-address.ld
//#LinkArgs:--section-start=.foo=0x600000
//#ExpectSym:orphan section=.foo
//#ExpectSym:matched section=.foo

//#Config:tdata:default
//#CompArgs:-DTEST_TDATA
//#LinkerScript:orphan-tdata.ld
//#ExpectProgramHeader:TLS sections=[.tdata,.tdata.orphan,.tbss,*]
//#ExpectSym:regular_tdata section=.tdata
//#ExpectSym:orphan_tdata section=.tdata.orphan
//#ExpectSym:uninit_tbss section=.tbss
//#ExpectSym:data section=.data

//#Config:tls:default
//#CompArgs:-DTEST_TLS
//#LinkerScript:orphan-tls.ld
//#ExpectProgramHeader:TLS sections=[.tdata,.tbss,*]
//#ExpectSym:initialized section=.tdata
//#ExpectSym:uninitialized section=.tbss

//#Config:tls_unplaced:default
//#CompArgs:-DTEST_TLS_UNPLACED
//#LinkerScript:orphan-tls-unplaced.ld
//#ExpectProgramHeader:TLS sections=[.tdata,.tbss,*]
//#ExpectSym:initialized section=.tdata
//#ExpectSym:uninitialized section=.tbss
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

//#Config:eh_frame_hdr:default
//#LinkerScript:eh-frame-hdr.ld
//#CompArgs:-fexceptions -fasynchronous-unwind-tables -DTEST_EH_FRAME_HDR
//#LinkArgs:--eh-frame-hdr
//#ExpectSection:.eh_frame_hdr
//#ExpectSym:foo section=.text
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

//#Config:custom_data:default
//#LinkerScript:custom-data.ld
//#CompArgs:-DTEST_CUSTOM_DATA
//#ExpectSym:custom section=.custom_data
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

//#Config:custom_data_loc:default
//#ReferenceLinkers:bfd,lld
//#LinkerScript:custom-data-loc.ld
//#CompArgs:-DTEST_CUSTOM_DATA
//#ExpectSym:custom section=.custom_data
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

//#Config:bss_before_data:default
//#ReferenceLinkers:lld
//#DiffEnabled:false
//#LinkerScript:bss-before-data.ld
//#CompArgs:-DTEST_BSS_BEFORE_DATA
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

//#Config:bss_before_data_runtime:default
//#Object:runtime.c
//#RunEnabled:true
//#LinkerScript:bss-before-data-runtime.ld
//#CompArgs:-DTEST_BSS_BEFORE_DATA_RUNTIME
//#ExpectSym:data section=.data
//#ExpectSym:bss section=.bss

#if defined(TEST_ORPHAN_DATA)
#include "../common/runtime.h"

int data_var = 42;
int __attribute__((section(".orphan_data"))) orphan_var = 100;
int bss_var;

void _start(void) {
  if (data_var != 42 || orphan_var != 100 || bss_var != 0) {
    exit_syscall(1);
  }
  orphan_var = 101;
  if (orphan_var != 101) {
    exit_syscall(2);
  }
  exit_syscall(42);
}

#elif defined(TEST_ADDRESS)

int data = 42;
int bss;

void _start(void) {}

#elif defined(TEST_EH_FRAME_HDR)

void foo(void) {}
int data = 42;
int bss;

void _start(void) {}

#elif defined(TEST_INIT_ARRAY)

void foo(void) {}
void (*init_funcs[])(void) __attribute__((section(".init_array"))) = {foo};
int data = 42;
int bss;

void _start(void) {}

#elif defined(TEST_CUSTOM_DATA)

void foo(void) {}
void (*init_funcs[])(void) __attribute__((section(".init_array"))) = {foo};
__attribute__((section(".custom_data"))) int custom = 1;
int data = 2;
int bss;

void _start(void) {}

#elif defined(TEST_BSS_BEFORE_DATA) || defined(TEST_BSS_BEFORE_DATA_RUNTIME)

#include "../common/runtime.h"

void foo(void) {}
void (*init_funcs[])(void) __attribute__((section(".init_array"))) = {foo};
int bss;
int data = 42;

void _start(void) {
#ifdef TEST_BSS_BEFORE_DATA_RUNTIME
  runtime_init();
  if (data != 42) {
    exit_syscall(1);
  }
  if (bss != 0) {
    exit_syscall(2);
  }
  exit_syscall(42);
#endif
}

#elif defined(TEST_LOC_COUNTER_DATA)

__attribute__((section(".orphan"))) int initialized = 7;
int uninitialized;

void _start(void) {}

#elif defined(TEST_LOC_COUNTER_TLS) || defined(TEST_TLS)

_Thread_local int initialized = 7;
_Thread_local int uninitialized;
int data = 42;

void _start(void) {}

#elif defined(TEST_TLS_UNPLACED)

_Thread_local int initialized = 7;
_Thread_local int uninitialized;
int data = 42;
int bss;

void _start(void) {}

#elif defined(TEST_TDATA)

_Thread_local int regular_tdata = 1;
__attribute__((section(".tdata.orphan"))) _Thread_local int orphan_tdata = 2;
_Thread_local int uninit_tbss;
int data = 42;

void _start(void) {}

#elif defined(TEST_MERGE_ADDRESS)

__attribute__((section(".foo"))) int orphan = 42;
__attribute__((section(".foo.other"))) int matched = 24;

void _start(void) {}

#endif
