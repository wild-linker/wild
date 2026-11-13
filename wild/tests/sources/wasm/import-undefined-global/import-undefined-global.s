//#LinkArgs: --import-undefined
//#ExpectError: undefined symbol: missing_global

    .globaltype missing_global, i32

    .globl _start
_start:
    .functype _start () -> ()
    global.get missing_global
    drop
    end_function
