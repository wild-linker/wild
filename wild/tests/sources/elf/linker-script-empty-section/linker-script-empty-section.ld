ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
}

SECTIONS {
    . = 0x400000 + SIZEOF_HEADERS;
    .text : {
        *(.text)
    } :text
    .empty_1 : {
        empty_1_start = .;
        *(.empty_1)
    }

    .eh_frame : {
        *(.eh_frame)
    }

    . = ALIGN(0x10000);
    .data : {
        KEEP(*(.data))
    } :data
    .empty_2 : {
        empty_2_start = .;
        *(.empty_2)
    }

    .empty_3 (INFO) : {
        empty_3_start = .;
        *(.empty_3)
    } :NONE

    .empty_4 (NOLOAD) : {
        empty_4_start = .;
        *(.empty_4)
    }

    .exec_only_sec : {
        KEEP(*(.exec_only_sec))
    }
    .empty_5 : {
        empty_5_start = .;
        *(.empty_5)
    }

    .write_only_sec : {
        KEEP(*(.write_only_sec))
    }
    .empty_6 : {
        empty_6_start = .;
        *(.empty_6)
    }

    .empty_7 : {
        *(.empty_7)
    }
}
