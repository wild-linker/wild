PROVIDE(provided_absolute = 0x1000);
PROVIDE_HIDDEN(provided_hidden_absolute = 0x2000);
PROVIDE(provided_symbol = provided_absolute);
PROVIDE(provided_expr1 = provided_symbol + provided_hidden_absolute);
PROVIDE(provided_expr2 = (0xffffffff80000000 + (((provided_absolute * 0x1000) + (provided_hidden_absolute * 0x100 - 1)) & ~(0x200000 - 1))));
PROVIDE(unreferenced_sym = 0x4000);
PROVIDE(unreferenced_sym2 = undefined_sym);
PROVIDE(referenced_sym = 0x5000);
PROVIDE(referenced_sym2 = referenced_sym + 0x30);

SECTIONS {
	PROVIDE(__sections_start = .);
    .text : {
        PROVIDE(__text_start = .);
        *(.text .text.*)
        PROVIDE(__text_end = .);
    }
	PROVIDE(__sections_mid = .);
    .data : {
        PROVIDE(__data_start = .);
        *(.data .data.*)
        PROVIDE(__data_end = .);
    }
	PROVIDE(__sections_end = .);
}

ASSERT(__sections_start == 0, "__sections_start != 0");
ASSERT(__sections_mid == __text_end, "__sections_mid != __text_end");
ASSERT(__sections_end == __data_end, "__sections_end != __data_end");
