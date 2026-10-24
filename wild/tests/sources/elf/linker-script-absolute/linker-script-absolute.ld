ENTRY(_start)

abs_global = ABSOLUTE(0x1000);
abs_global_calc = ABSOLUTE(0x1000 + 0x500);

SECTIONS
{
    . = 0x400000;
    .text 0x500000 : {
        text_start = .;
        ASSERT(ABSOLUTE(0) != ., "ABSOLUTE(0) != .");
        a = . + 0x20;
        b = . + 0x10;

        abs_const = ABSOLUTE(0x500);
        abs_dot = ABSOLUTE(.);
        abs_sym_a = ABSOLUTE(a);
        abs_sum_two = ABSOLUTE(ABSOLUTE(a) + ABSOLUTE(b));
        abs_max_func = ABSOLUTE(MAX(a, b));
        abs_nested = ABSOLUTE(ABSOLUTE(a));
        absolute_aligned = ABSOLUTE(ALIGN(16));
        aligned_absolute = ALIGN(ABSOLUTE(16), 16);
        absolute_plus = ABSOLUTE(16) + 8;
        absolute_plus_comparison = ABSOLUTE(16) + (1 == 1);
        sum_absolute = ABSOLUTE(16) + ABSOLUTE(8);
        sum_absolute_addresses = ABSOLUTE(text_start) + ABSOLUTE(text_start);
        absolute_ternary = 1 ? ABSOLUTE(24) : ABSOLUTE(32);
        ASSERT(absolute_ternary == 24, "ternary must preserve absolute symbol classification");
        absolute_max = MAX(ABSOLUTE(16), ABSOLUTE(24));
        ASSERT(absolute_max == 24, "MAX must preserve absolute symbol classification");
        abs_max_mixed = MAX(ABSOLUTE(16), text_start);
        abs_max_mixed_rev = MAX(text_start, ABSOLUTE(16));

        ternary_taken_rel = a ? text_start : ABSOLUTE(32);
        ternary_taken_abs = !a ? text_start : ABSOLUTE(32);
        ternary_num_vs_abs = 1 ? 24 : ABSOLUTE(32);
        ternary_abs_vs_num = 1 ? ABSOLUTE(24) : 32;

        abs_land = ABSOLUTE(1) && ADDR(.text);
        abs_lor = ABSOLUTE(0) || ADDR(.text);
        abs_lnot = !ABSOLUTE(ADDR(.text));

        ASSERT(abs_land == text_start + 1, "abs_land inside failed");
        ASSERT(abs_lor == text_start + 1, "abs_lor inside failed");
        ASSERT(abs_lnot == 0, "abs_lnot inside failed");

        padding = ABSOLUTE(16);
        . += padding;
        after_padding = .;

        absolute_last = ABSOLUTE(. + ABSOLUTE(0));
        absolute_first = ABSOLUTE(ABSOLUTE(0) + .);

        ASSERT(ABSOLUTE(.) == ., "absolute location must equal relative location");
        ASSERT(. == ABSOLUTE(.), "absolute location must equal relative location");

        ASSERT(text_start == ABSOLUTE(text_start), "ABSOLUTE(text_start) == text_start");
        ASSERT(ABSOLUTE(text_start) == text_start, "ABSOLUTE(text_start) == text_start");
        ASSERT(a == ABSOLUTE(text_start) + 0x20, "a failed");
        ASSERT(ABSOLUTE(text_start) + 0x20 == a, "absolute+const == relative");

        ASSERT(ABSOLUTE(ADDR(.text)) == text_start, "ABSOLUTE(constant VMA) == relative");
        ASSERT(text_start == ABSOLUTE(ADDR(.text)), "relative == ABSOLUTE(constant VMA)");
        ASSERT(ABSOLUTE(0) != text_start, "ABSOLUTE(0) != text_start");
        ASSERT(text_start != ABSOLUTE(0), "text_start != ABSOLUTE(0)");

        ASSERT(abs_global == 0x1000, "abs_global failed");
        ASSERT(abs_global_calc == 0x1500, "abs_global_calc failed");
        ASSERT(abs_const == 0x500, "abs_const failed");

        ASSERT(abs_dot == ABSOLUTE(text_start), "abs_dot failed");
        ASSERT(abs_dot != text_start, "abs_dot failed");
        ASSERT((abs_dot * 2) == (ABSOLUTE(text_start) * 2), "abs_dot failed");
        ASSERT((ABSOLUTE(text_start) * 2) == (abs_dot * 2), "abs_dot failed");
        ASSERT((abs_dot * 2) != (text_start * 2), "abs_dot failed");

        ASSERT(abs_sym_a == ABSOLUTE(text_start) + 0x20, "abs_sym_a failed");
        ASSERT(abs_sym_a != text_start + 0x20, "abs_sym_a failed");

        ASSERT(abs_sum_two == ABSOLUTE(text_start) * 2 + 0x30, "abs_sum_two failed");
        ASSERT(abs_sum_two != text_start * 2 + 0x30, "abs_sum_two failed");

        ASSERT(abs_max_func == ABSOLUTE(text_start) + 0x20, "abs_max_func failed");
        ASSERT(ABSOLUTE(text_start) + 0x20 == abs_max_func, "abs_max_func failed");
        ASSERT(abs_max_func != text_start + 0x20, "abs_max_func failed");

        ASSERT(abs_nested == ABSOLUTE(text_start) + 0x20, "abs_nested failed");

        ASSERT(absolute_aligned == ABSOLUTE(text_start), "absolute_aligned failed");
        ASSERT(absolute_aligned != text_start, "absolute_aligned failed");

        ASSERT(absolute_plus == 24, "arithmetic must preserve absolute symbol classification");

        ASSERT(after_padding == text_start + 0x10, "after_padding failed");

        ASSERT(absolute_first == absolute_last, "order does not affect value");

        *(.text)
    }

    . = 0x600000;
    .data : {
        data_start = .;

        e = . + 0x30;

        ASSERT(abs_const == 0x500, "abs_const failed");

        ASSERT(e + a == ABSOLUTE(text_start + data_start) + 0x20 + 0x30, "absolute addressing failed");

        KEEP(*(.data .data.*))
    }

    .bss : {
        *(.bss .bss.*)
    }
}

ASSERT(absolute_aligned == text_start, "absolute_aligned failed");
ASSERT(absolute_plus_comparison == 17, "comparison result must act as a plain number");
ASSERT(aligned_absolute == 16, "ALIGN must preserve absolute value");
ASSERT(sum_absolute == text_start + 24, "sum of absolute operands must be section relative");
ASSERT(sum_absolute_addresses == text_start * 3, "sum of absolute addresses must be section relative");

ASSERT(abs_land == text_start + 1, "abs_land global failed");
ASSERT(abs_lor == text_start + 1, "abs_lor global failed");
ASSERT(abs_lnot == 0, "abs_lnot global failed");
