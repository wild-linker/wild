//#Config:after-stack
//#LinkArgs:--stack-first -z stack-size=65536 --global-base=131072 --export=__global_base
//#ExpectSym:__global_base address=131072

//#Config:no-stack-first
//#LinkArgs:--no-stack-first -z stack-size=65536 --global-base=4096 --export=__global_base
//#RequiresLinkerFlags:--no-stack-first
//#ExpectSym:__global_base address=4096

//#Config:zero
//#LinkArgs:--stack-first -z stack-size=65536 --global-base=0 --export=__global_base
//#ExpectSym:__global_base address=65536

//#Config:below-stack
//#LinkArgs:--stack-first -z stack-size=65536 --global-base=2048
//#ExpectError:--global-base cannot be less than stack size when --stack-first is used

extern char __global_base;
int payload = 1;

void _start(void) {
  unsigned long base = (unsigned long)&__global_base;
  if ((unsigned long)&payload < base || payload != 1) {
    __builtin_trap();
  }
}
