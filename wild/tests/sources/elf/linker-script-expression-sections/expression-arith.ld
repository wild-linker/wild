ENTRY(_start)
abs_offset = 0x20;

SECTIONS {
    .data 0x20000 : {
        data_start = .;
        KEEP(*(.data*))
        data_end = .;
    }
    mid_align = ALIGN(0x200);
    mid_align_dot = ALIGN(., 0x200);
    mid_align_expr = ALIGN(. + 1, 0x100);
    mid_align_num = ALIGN(33, 16);
    mid_abs = 32;
    mid_loc = . + 21;
    .text 0x10000 : {
        sec_start = .;
        in_sec_add = sec_start + 0x10;
        in_sec_sub = (sec_start + 0x20) - 0x10;
        in_sec_mul = (sec_start + 0x10) * 2;
        sec_end = sec_start + 0x100;
        in_sec_add_rev = 0x1000 + sec_start;
        in_sec_sub_rev = 0x1000 - sec_start;
        in_sec_loadaddr_sub = LOADADDR(.text) - 0x1000;
        in_sec_loadaddr_mul = LOADADDR(.text) * 2;
        in_sec_loadaddr_and = LOADADDR(.text) & 0xffff;
        in_sec_loadaddr_diff = LOADADDR(.text) - sec_end;
        in_sec_loadaddr_diff_rev = sec_end - LOADADDR(.text);
        in_sec_loadaddr_add = sec_end + LOADADDR(.text);
        in_sec_cross_diff = data_start - sec_start;
        in_sec_cross_add = data_start + sec_start;
        in_sec_cross_ref = data_start;
        in_sec_neg = -sec_start;
        in_sec_not = ~sec_start;
        in_sec_abs_alias = abs_offset;
        in_sec_align = ALIGN(0x100);
        in_sec_align_expr = ALIGN(sec_start + 1, 0x100);
        *(.text*)
    }
}

out_add = sec_start + 0x1000;
out_add_rev = 0x1000 + sec_start;
out_sub = sec_start - 0x1000;

out_sub_rev = 0x30000 - sec_start;

out_mul = sec_start * 2;
out_div = sec_start / 2;
out_shift = sec_start >> 1;

out_diff = sec_end - sec_start;

out_rel_plus_abs = sec_start + abs_offset;
out_loadaddr_and = LOADADDR(.text) & 0xffff;
out_loadaddr_diff_rev = sec_end - LOADADDR(.text);
out_loadaddr_add = sec_end + LOADADDR(.text);
out_cross_diff = data_start - sec_start;
out_cross_add = data_start + sec_start;
out_neg = -sec_start;
out_not = ~sec_start;
out_abs_alias = abs_offset;
out_align_expr = ALIGN(sec_start + 1, 0x100);
