ENTRY(_start)

zero = 0;
one = 1;

SECTIONS {
    .text : ALIGN((0x2 << 3)) {
        *(.text*)
        symbol3 = 0x3000;
        symbol13 = .;
        symbol13_other = . + 0x10;
        symbol13_text_offset = symbol13 + 0x20;
    }
    symbol11 = 0x100;
    symbol11_assert = ASSERT(symbol11 == 0x100, "symbol11 must be 0x100");
    symbol11 -= 0x20;
    symbol11 &= 0x80;
    ASSERT(symbol11 == 0x80, "symbol11 must be 0x80")
    .data : AT(ADDR(.data) + 0x10000) {
        data_start = .;
        *(.data*)
        symbol10 = 0x100;
        symbol10_assert = ASSERT(symbol10 == 0x100, "symbol10 must be 0x100");
        symbol10 *= 0x100;
        symbol10 ^= 0x40;
        ASSERT(symbol10 == 0x10040, "symbol10 must be 0x10040");
        ASSERT(data_start == ADDR(.data), "data_start must be ADDR(.data)");
        ASSERT(data_start + 0x10000 == LOADADDR(.data), "data_start + 0x10000 must be LOADADDR(.data)");
    }
    .bss : AT(ADDR(.bss) + 0x10000) { *(.bss*) }
}

/* A trivially true assertion - should always pass */
ASSERT(1 == 1, "1 == 1 must always be true");

/* Arithmetic assertion */
ASSERT(1 + 1 == 2, "arithmetic must work");
ASSERT(12 / 10 == 1, "pos / pos division");
ASSERT(-50 / 10 == -5, "neg / pos division failed");
ASSERT(60 / -10 == -6, "pos / neg division failed");
ASSERT(-40 / -10 == 4, "neg / neg division failed");
ASSERT(12 % 10 == 2, "pos % pos modulo failed");
ASSERT(-5 % 10 == -5, "neg % pos modulo failed");
ASSERT(13 % -10 == 3, "pos % neg modulo failed");
ASSERT(-12 % -10 == -2, "neg % neg modulo failed");

/* SIZEOF(.text) should be > 0 since _start is in it */
ASSERT(SIZEOF(.text) > 0, "text section must be non-empty");

/* Realistic check: full evaluator path - function -> arithmetic -> comparison */
ASSERT(SIZEOF(.text) + SIZEOF(.data) < 0x100000, "image too big");

symbol1 = 0x1000;
symbol2 = 0x2000;

ASSERT(symbol1 == 0x1000, "symbol1 must be 0x1000");
ASSERT(symbol1 + symbol2 == 0x3000, "symbol1 + symbol2 must be 0x3000");
ASSERT(symbol1 + symbol2 != symbol1 - symbol2, "symbol1 + symbol2 must not be symbol1 - symbol2");
ASSERT(0xffffffff81000000 == (0xffffffff80000000 + (((symbol1 * 0x1000) + (symbol2 * 0x100 - 1)) & ~(0x200000 - 1))), "Incorrect Value");
ASSERT((5 > 4 ? !1 ? 3 : 5 : 4) == 5, "5 > 4 ? !1 ? 3 : 5 : 4 must be 5");
ASSERT((symbol1 > symbol2 ? symbol1 : symbol2) == MAX(symbol1, symbol2), "Expected MAX(symbol1, symbol2)");
ASSERT(DEFINED(symbol1), "symbol1 must be defined");
ASSERT(DEFINED(symbol3), "symbol3 must be defined");
ASSERT(DEFINED(symbol4), "symbol4 must be defined");
ASSERT(!DEFINED(symbol5), "symbol5 should not be defined");
ASSERT(DEFINED(symbol6), "symbol6 must be defined");
ASSERT(DEFINED(symbol7), "symbol7 must be defined");
ASSERT(!DEFINED(symbol8), "symbol8 should not be defined");

symbol9 = 0x8;
symbol9 += 0x10;
ASSERT(symbol9 == 0x18, "symbol9 must be 0x18");
symbol9 /= 0x2;
symbol9 <<= 0x3;
symbol9 |= 0x100;
ASSERT(symbol9 == 0x160, "symbol9 must be 0x160");

symbol12 = ASSERT(ASSERT(0x5000 < 0x10000, "0x5000 < 0x10000 must be true") == 1, "ASSERT must return 1");
. = ASSERT(symbol12 == 1, "must be 1");
ASSERT(ALIGN(0x38, 16) == 0x40, "ALIGN value is incorrect");
ASSERT(ALIGN(0x38, 18) == 0x48, "ALIGN value is incorrect");

ASSERT(ALIGNOF(.text) == 0x10, "ALIGNOF value is incorrect");

symbol14 = 0x1000;
symbol13a = one ? symbol13 : symbol14;
symbol14a = zero ? symbol13 : symbol14;
ASSERT(symbol13a == symbol13, "1 ? symbol13 : symbol14 must be symbol13");
ASSERT(symbol14a == symbol14, "0 ? symbol13 : symbol14 must be symbol14");

min_same_sec = MIN(symbol13, symbol13_other);
max_same_sec = MAX(symbol13, symbol13_other);
min_diff_sec = MIN(symbol13, data_start);
max_diff_sec = MAX(symbol13, data_start);
min_with_abs = MIN(symbol13, 0x10);
max_with_abs = MAX(symbol13, 0x10);

ASSERT(min_same_sec == symbol13, "min_same_sec is incorrect");
ASSERT(max_same_sec == symbol13_other, "max_same_sec is incorrect");
ASSERT(min_diff_sec == symbol13, "min_diff_sec is incorrect");
ASSERT(max_diff_sec == data_start, "max_diff_sec is incorrect");
ASSERT(min_with_abs == 0x10, "min_with_abs is incorrect");
ASSERT(max_with_abs == symbol13, "max_with_abs is incorrect");

symbol13_1 = symbol13;
symbol13_2 = symbol13_1;
symbol13_3 = symbol13_2;

diff_same_sec = symbol13_other - symbol13;
ASSERT(diff_same_sec == 0x10, "diff_same_sec must be 0x10");

ASSERT(symbol13 > 0x10, "text address should be greater than 0x10");

sym_log_and_true = symbol13 && symbol14;
sym_log_and_false = symbol13 && zero;
sym_log_or = zero || symbol13;
sym_log_not = !symbol14;
ASSERT(sym_log_and_true == 1, "sym_log_and_true incorrect");
ASSERT(sym_log_and_false == 0, "sym_log_and_false incorrect");
ASSERT(sym_log_or == 1, "sym_log_or incorrect");
ASSERT(sym_log_not == 0, "sym_log_not incorrect");

sym_bit_and = symbol14 & 0xff00;
sym_neg = -symbol14;
ASSERT(sym_bit_and == 0x1000, "sym_bit_and incorrect");
ASSERT(sym_neg == -0x1000, "sym_neg incorrect");
