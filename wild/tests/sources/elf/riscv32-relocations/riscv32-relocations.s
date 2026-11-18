/*
//#Arch: riscv32
//#LinkArgs: -no-pie
*/

.option norelax

.text
.globl _start

_start:
    call foo
    lw a0, val
    lw a1, ptr
    lw a1, 0(a1)
    sub a0, a0, a1
    sw a0, store_loc, t0
    lw a2, store_loc
    bne a0, a2, fail
    jal x1, bar
    lw a3, diff_word
    addi a3, a3, -4
    bne a3, zero, fail
    addi a0, a0, 42
done:
    li a7, 93
    ecall

fail:
    li a0, 1
    j done

.globl foo
foo:
    ret

bar:
    ret

.data
val:
    .word 42
ptr:
    .word val
store_loc:
    .word 0
diff_word:
    .word ptr - val
