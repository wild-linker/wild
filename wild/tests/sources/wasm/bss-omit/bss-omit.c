//#Config:default
//#ExpectDataSegmentCount: 1
//#ExpectDataSegmentName: 0=.data

//#Config:import-memory
//#CompArgs: -mno-bulk-memory
//#LinkArgs: --stack-first --import-memory --initial-memory=131072 --max-memory=196608
//#WasmPreload: env=dirty-provider.wat
//#ExpectDataSegmentCount: 2
//#ExpectDataSegmentName: 0=.data
//#ExpectDataSegmentName: 1=.bss

//#Config:shared-memory
//#CompArgs: -matomics -mbulk-memory
//#LinkArgs: --stack-first --shared-memory --max-memory=131072
//#ExpectSharedMemory: true
//#ExpectSection: Start
//#ExpectSection: DataCount
//#ExpectDataSegmentCount: 1
//#ExpectDataSegmentName: 0=.data

//#Config:shared-memory-import
//#CompArgs: -matomics -mbulk-memory
//#LinkArgs: --stack-first --shared-memory --import-memory --initial-memory=131072 --max-memory=196608
//#WasmPreload: env=dirty-shared-provider.wat
//#ExpectSection: Start
//#ExpectSection: DataCount
//#ExpectDataSegmentCount: 1
//#ExpectDataSegmentName: 0=.data

extern char __data_end;

volatile int init_val = 42;
volatile unsigned int big_bss[1024];

void _start(void) {
  if (init_val != 42) {
    __builtin_trap();
  }
  for (int i = 0; i < 1024; i++) {
    if (big_bss[i] != 0) {
      __builtin_trap();
    }
  }
  if ((unsigned long)&big_bss[1024] > (unsigned long)&__data_end) {
    __builtin_trap();
  }
}
