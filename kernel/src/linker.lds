OUTPUT_ARCH(riscv)
ENTRY(_low_entry)
INIT_ADDRESS = 0x80200000;

BASE_ADDRESS = 0xfffffff000200000;

SECTIONS
{
    . = BASE_ADDRESS;

    .init : AT(INIT_ADDRESS) {
        __boot_start = .;
        *(.init.boot)
        __boot_end = ABSOLUTE(.);
    }

    __kernel_start = .;

    __text_start = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    __text_end = .;
    __rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    __rodata_end = .;
    __data_start = .;
    .data : {
        *(.data.boot_page_table)
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    __data_end = .;
    .bss : {
        *(.bss.stack)
        __bss_start = .;

        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    __bss_end = .;
    __kernel_end = .;
}